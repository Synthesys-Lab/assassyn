use proc_macro::TokenStream;
use syn::{braced, parse::Parse, Ident};

use crate::{codegen::ExprToType, Instruction};

pub(crate) struct TypeParser {
  pub(crate) ty: TokenStream,
}

impl Parse for TypeParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    match ExprToType::parse(input.clone()) {
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

pub(crate) struct BodyParser {
  pub(crate) stmts: Vec<Instruction>,
}

impl Parse for BodyParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = braced!(content in input);
    let mut stmts = Vec::new();
    while !content.is_empty() {
      if content.peek(syn::Ident) {
        let id = content.parse::<Ident>()?;
        // <id> = <expr>
        if content.peek(syn::Token![=]) {
          content.parse::<syn::Token![=]>()?;
          let assign = content.parse::<syn::Expr>()?;
          stmts.push(Instruction::Assign((id, assign)));
        } else if content.peek(syn::token::Bracket) {
          // <id>[<expr>] = <expr>
          let idx;
          syn::bracketed!(idx in content);
          let idx = idx.parse::<syn::Expr>()?;
          content.parse::<syn::Token![=]>()?;
          let assign = content.parse::<syn::Expr>()?;
          stmts.push(Instruction::ArrayAssign((id, idx, assign)));
        } else {
          return Err(syn::Error::new(
            content.span(),
            "Expected an assignment or an expression",
          ));
        }
      }
      content.parse::<syn::Token![;]>()?;
    }
    Ok(BodyParser { stmts })
  }
}
