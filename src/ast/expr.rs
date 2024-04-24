use eir::ir::DataType;
use syn::{parenthesized, parse::Parse};

pub(crate) enum ExprTerm {
  Ident(syn::Ident),
  Const((DType, syn::LitInt)),
  StrLit(syn::LitStr),
}

impl Parse for ExprTerm {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(syn::LitStr) {
      let lit = input.parse::<syn::LitStr>()?;
      Ok(ExprTerm::StrLit(lit))
    } else if let Some(_) = input.cursor().ident() {
      let id = input.clone().parse::<syn::Ident>()?;
      Ok(ExprTerm::Ident(id))
    } else if let Some(_) = input.cursor().literal() {
      let lit = input.parse::<syn::LitInt>()?;
      let ty = if input.peek(syn::Token![.]) {
        input.parse::<syn::Token![.]>()?;
        input.parse::<DType>()?
      } else {
        DType {
          span: lit.span(),
          dtype: DataType::int(32),
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
      match id.to_string().as_str() {
        "default" => {
          let default_value = input.parse::<ExprTerm>()?;
          let mut cases = Vec::new();
          while !input.peek(syn::Token![;]) {
            input.parse::<syn::Token![.]>()?; // Consume "."
            input.parse::<syn::Ident>()?; // Consume "case"
            let cond = input.parse::<ExprTerm>()?;
            input.parse::<syn::Token![,]>()?; // Consume ","
            let value = input.parse::<ExprTerm>()?;
            cases.push((cond, value));
          }
          return Ok(Expr::Select((default_value, cases)));
        }
        "slice" => {
          input.parse::<syn::Token![.]>()?; // Consume "."
          input.parse::<syn::Ident>()?; // Consume "slice"
          let content;
          parenthesized!(content in input);
          let l = content.parse::<ExprTerm>()?;
          content.parse::<syn::Token![,]>()?; // Consume ","
          let r = content.parse::<ExprTerm>()?;
          return Ok(Expr::Slice((tok, l, r)));
        }
        _ => {}
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
    if content.is_empty() {
      Ok(Expr::Unary((a, operator)))
    } else {
      let b = content.parse::<ExprTerm>()?;
      Ok(Expr::Binary((a, operator, b)))
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
    let span = input.cursor().span().clone();
    let tyid = input.parse::<syn::Ident>()?;
    match tyid.to_string().as_str() {
      "int" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::int(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "uint" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::uint(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "module" => {
        let args;
        parenthesized!(args in input);
        let parsed_args = args.parse_terminated(DType::parse, syn::Token![,])?;
        Ok(DType {
          span,
          dtype: DataType::module(parsed_args.iter().map(|x| x.dtype.clone().into()).collect()),
        })
      }
      _ => {
        return Err(syn::Error::new(
          tyid.span(),
          format!("[CG.Type] Unsupported type: {}", tyid.to_string()),
        ));
      }
    }
  }
}
