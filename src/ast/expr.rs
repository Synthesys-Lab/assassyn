use eir::frontend::DataType;
use syn::{parenthesized, parse::Parse};

pub(crate) enum Expr {
  Ident(syn::Ident),
  Const((DType, syn::LitInt)),
}

impl Parse for Expr {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if let Some(_) = input.cursor().ident() {
      let id = input.clone().parse::<syn::Ident>()?;
      Ok(Expr::Ident(id))
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
      Ok(Expr::Const((ty, lit)))
    } else {
      Err(syn::Error::new(
        input.span(),
        "Expected identifier or literal",
      ))
    }
  }
}

#[derive(Clone)]
pub(crate) struct DType {
  pub(crate) span: proc_macro2::Span,
  pub(crate) dtype: eir::frontend::DataType,
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
          dtype: eir::frontend::DataType::int(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "uint" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: eir::frontend::DataType::uint(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "module" => {
        let args;
        parenthesized!(args in input);
        let parsed_args = args.parse_terminated(DType::parse, syn::Token![,])?;
        Ok(DType {
          span,
          dtype: eir::frontend::DataType::module(
            parsed_args.iter().map(|x| x.dtype.clone().into()).collect(),
          ),
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
