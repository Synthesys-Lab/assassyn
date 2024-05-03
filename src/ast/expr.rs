use eir::ir::DataType;
use syn::{parenthesized, parse::Parse, punctuated::Punctuated, Token};

use super::node::ArrayAccess;

pub(crate) enum ExprTerm {
  Ident(syn::Ident),
  Const((DType, syn::LitInt)),
  StrLit(syn::LitStr),
}

pub(crate) struct ModuleAttrs {
  pub(crate) attrs: Punctuated<syn::Ident, Token![,]>,
}

impl Parse for ModuleAttrs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut attrs = Punctuated::new();
    loop {
      if !input.peek(Token![#]) {
        break;
      }
      input.parse::<Token![#]>()?;
      let raw_attr = input.parse::<syn::Ident>()?;
      attrs.push_value(match raw_attr.to_string().as_str() {
        "optnone" | "explicit_pop" => raw_attr.clone(),
        _ => {
          return Err(syn::Error::new(
            raw_attr.span(),
            format!(
              "{}:{}: Unsupported attribute: \"{}\"",
              file!(),
              line!(),
              raw_attr
            ),
          ))
        }
      });
      if !input.peek(Token![,]) {
        break;
      }
      attrs.push_punct(input.parse::<Token![,]>()?);
    }
    Ok(ModuleAttrs { attrs })
  }
}

impl ExprTerm {
  pub(crate) fn span(&self) -> proc_macro2::Span {
    match self {
      ExprTerm::Ident(id) => id.span(),
      ExprTerm::Const((_, lit)) => lit.span(),
      ExprTerm::StrLit(lit) => lit.span(),
    }
  }
}

impl Parse for ExprTerm {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(syn::LitStr) {
      let lit = input.parse::<syn::LitStr>()?;
      Ok(ExprTerm::StrLit(lit))
    } else if input.cursor().ident().is_some() {
      let id = input.parse::<syn::Ident>().unwrap_or_else(|_| {
        panic!(
          "{}:{}: Failed to parse identifier in ExprTerm",
          file!(),
          line!()
        )
      });
      Ok(ExprTerm::Ident(id))
    } else if input.cursor().literal().is_some() {
      let lit = input.parse::<syn::LitInt>()?;
      let ty = if input.peek(syn::Token![.]) {
        input.parse::<syn::Token![.]>()?;
        input.parse::<DType>()?
      } else {
        DType {
          span: lit.span(),
          dtype: DataType::int_ty(32),
        }
      };
      Ok(ExprTerm::Const((ty, lit)))
    } else {
      Err(syn::Error::new(
        input.span(),
        "Expected identifier or literal",
      ))
    }
  }
}

/// The left value of an assignment.
pub(crate) enum LValue {
  IdentList(Punctuated<syn::Ident, Token![,]>),
  Ident(syn::Ident),
  ArrayAccess(ArrayAccess),
}

impl Parse for LValue {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(syn::Ident) {
      if input.peek2(syn::token::Bracket) {
        let aa = input
          .parse::<ArrayAccess>()
          .unwrap_or_else(|_| panic!("{}:{}: Failed to parse array access", file!(), line!()));
        Ok(LValue::ArrayAccess(aa))
      } else {
        let mut idents = Punctuated::new();
        loop {
          idents.push_value(
            input
              .parse::<syn::Ident>()
              .unwrap_or_else(|_| panic!("{}:{}: Failed to parse identifier", file!(), line!())),
          );
          if !input.peek(syn::Token![,]) {
            break;
          }
          idents.push_punct(input.parse::<syn::Token![,]>()?);
          if !input.peek(syn::Ident) {
            break;
          }
        }
        if idents.len() == 1 && !idents.trailing_punct() {
          Ok(LValue::Ident(idents.first().unwrap().clone()))
        } else {
          Ok(LValue::IdentList(idents))
        }
      }
    } else {
      Err(syn::Error::new(input.span(), "Expected an identifier"))
    }
  }
}

pub(crate) enum Expr {
  // ExprTerm . syn::Ident ( ExprTerm ): a.add(b)
  Binary((ExprTerm, syn::Ident, ExprTerm)),
  // ExprTerm . syn::Ident ( ): a.flip()
  Unary((ExprTerm, syn::Ident)),
  // "default" ExprTerm . "case" ( ExprTerm, ExprTerm )
  //                    . "case" ( ExprTerm, ExprTerm ) *
  Select((ExprTerm, Vec<(ExprTerm, ExprTerm)>)),
  // ExprTerm . slice ( ExprTerm, ExprTerm )
  Slice((ExprTerm, ExprTerm, ExprTerm)),
  // ExprTerm
  Term(ExprTerm),
}

impl Parse for Expr {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tok = input.parse::<ExprTerm>()?;
    if let ExprTerm::Ident(id) = &tok {
      if *id == "default" {
        let default_value = input.parse::<ExprTerm>()?;
        let mut cases = Vec::new();
        while !input.peek(syn::Token![;]) {
          input.parse::<syn::Token![.]>()?; // Consume "."
          input.parse::<syn::Ident>()?; // Consume "case"
          let content;
          parenthesized!(content in input);
          let cond = content.parse::<ExprTerm>()?;
          content.parse::<syn::Token![,]>().expect("Expect a \",\""); // Consume ","
          let value = content.parse::<ExprTerm>()?;
          cases.push((cond, value));
        }
        return Ok(Expr::Select((default_value, cases)));
      }
    }
    if !input.peek(syn::Token![.]) {
      return Ok(Expr::Term(tok));
    }
    let a = tok;
    input.parse::<syn::Token![.]>()?; // Consume "."
    let operator = input.parse::<syn::Ident>()?;
    let content;
    parenthesized!(content in input);
    match operator.to_string().as_str() {
      "slice" => {
        let l = content.parse::<ExprTerm>()?;
        content.parse::<syn::Token![,]>()?; // Consume ","
        let r = content.parse::<ExprTerm>()?;
        Ok(Expr::Slice((a, l, r)))
      }
      // TODO(@were): Deprecate pop, make it opaque to users.
      "flip" | "pop" | "valid" | "peek" => Ok(Expr::Unary((a, operator))),
      "add" | "mul" | "sub" | "igt" | "ilt" | "ige" | "ile" | "eq" | "bitwise_and" => {
        let b = content.parse::<ExprTerm>()?;
        Ok(Expr::Binary((a, operator, b)))
      }
      _ => Err(syn::Error::new(
        operator.span(),
        format!(
          "{}:{}: Unsupported operator: \"{}\"",
          file!(),
          line!(),
          operator
        ),
      )),
    }
  }
}

#[derive(Clone)]
pub(crate) struct DType {
  pub(crate) span: proc_macro2::Span,
  pub(crate) dtype: DataType,
}

impl Parse for DType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let span = input.cursor().span();
    let tyid = input.parse::<syn::Ident>()?;
    match tyid.to_string().as_str() {
      "int" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::int_ty(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "uint" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::uint_ty(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "module" => {
        let args;
        parenthesized!(args in input);
        let parsed_args = args.parse_terminated(DType::parse, syn::Token![,])?;
        Ok(DType {
          span,
          dtype: DataType::module(parsed_args.iter().map(|x| x.dtype.clone()).collect()),
        })
      }
      _ => Err(syn::Error::new(
        tyid.span(),
        format!("[CG.Type] Unsupported type: {}", tyid),
      )),
    }
  }
}
