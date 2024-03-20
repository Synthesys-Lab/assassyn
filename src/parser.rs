use proc_macro::TokenStream;
use syn::parse::Parse;

use crate::codegen;

pub(crate) struct TypeParser {
  ty: TokenStream,
}

impl Parse for TypeParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    match input.cursor().ident() {
      Some((id, _)) => match id.to_string().as_str() {
        "int" | "uint" => {
          return Ok(TypeParser {
            ty: codegen::expr_to_type(input.parse::<syn::Expr>()?)?,
          })
        }
        _ => {
          return Err(syn::Error::new(
            id.span(),
            format!("Unsupported type: {}", id.to_string()),
          ));
        }
      },
      None => unreachable!(),
    }
  }
}

pub(crate) struct Argument {
  id: syn::Ident,
  ty: TokenStream,
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
