use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{braced, parse::Parse, ExprStruct, Ident};

use crate::codegen::{EmitIDOrConst, EmitType};

pub(crate) struct TypeParser {
  pub(crate) ty: TokenStream,
}

pub(crate) enum Instruction {
  Assign((syn::Ident, syn::Expr)),
  ArrayAssign((ArrayAccess, syn::Expr)),
  ArrayRead((syn::Ident, ArrayAccess)),
  AsyncCall((syn::Ident, Vec<(syn::Ident, syn::Expr)>)),
}

impl Parse for TypeParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    match EmitType::parse(input.clone()) {
      Ok(ty) => Ok(TypeParser { ty: ty.0 }),
      Err(err) => Err(err),
    }
  }
}

pub(crate) struct Argument {
  pub(crate) id: syn::Ident,
  pub(crate) ty: TokenStream,
}

impl Parse for Argument {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input
      .parse::<syn::Ident>()
      .map_err(|e| syn::Error::new(e.span(), "Expected a port id"))?;
    let _ = input
      .parse::<syn::Token![:]>()
      .map_err(|e| syn::Error::new(e.span(), "Expected : to specify the type of the port"))?;
    let ty = input.parse::<TypeParser>()?.ty;
    Ok(Argument { id, ty })
  }
}

pub(crate) struct Body {
  pub(crate) stmts: Vec<Instruction>,
}

pub(crate) struct ArrayAccess {
  pub(crate) id: syn::Ident,
  pub(crate) idx: TokenStream,
}

impl Parse for ArrayAccess {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input.parse::<syn::Ident>()?;
    let idx;
    syn::bracketed!(idx in input);
    let idx = idx.parse::<EmitIDOrConst>()?.0;
    Ok(ArrayAccess { id, idx })
  }
}

impl Parse for Body {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = braced!(content in input);
    let mut stmts = Vec::new();
    while !content.is_empty() {
      if content.peek(syn::token::Async) {
        // async <func-id> { <id>: <expr>, ... }
        content.parse::<syn::token::Async>()?;
        let args = content.parse::<ExprStruct>()?;
        let func = syn::parse::<Ident>(args.path.into_token_stream().into())?;
        let fields = args
          .fields
          .iter()
          .map(|x| match &x.member {
            syn::Member::Named(id) => (id.clone(), x.expr.clone()),
            _ => panic!("Expected a named member"),
          })
          .collect::<Vec<_>>();
        stmts.push(Instruction::AsyncCall((func, fields)));
      } else if content.peek(syn::Ident) {
        // Parse non-keyword-leading statements
        if content.peek2(syn::token::Bracket) {
          // <id>[<expr>] = <expr>
          let aa = content.parse::<ArrayAccess>()?;
          content.parse::<syn::Token![=]>()?;
          let right = content.parse::<syn::Expr>()?;
          stmts.push(Instruction::ArrayAssign((aa, right)));
        } else {
          // <id> = <expr>
          let id = content.parse::<Ident>()?;
          if content.peek(syn::Token![=]) {
            content.parse::<syn::Token![=]>()?;
            // to handle the expression in k = a[0.int::<32>]
            if content.peek(syn::Ident) && content.peek2(syn::token::Bracket) {
              let aa = content.parse::<ArrayAccess>()?;
              stmts.push(Instruction::ArrayRead((id, aa)));
            } else {
              let assign = content.parse::<syn::Expr>()?;
              stmts.push(Instruction::Assign((id, assign)));
            }
          } else {
            return Err(syn::Error::new(
              content.span(),
              "Expected an assignment or an expression",
            ));
          }
        }
      }
      content.parse::<syn::Token![;]>()?;
    }
    Ok(Body { stmts })
  }
}
