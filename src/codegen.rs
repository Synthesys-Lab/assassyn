use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

pub(crate) fn expr_to_type(raw: syn::Expr) -> syn::Result<TokenStream> {
  if let syn::Expr::Path(path) = raw {
    assert_eq!(path.path.segments.len(), 1);
    let stripped = path.path.segments.iter().next().unwrap();
    let id = stripped.ident.clone();
    match &stripped.arguments {
      syn::PathArguments::AngleBracketed(args) => {
        assert_eq!(args.args.len(), 1);
        let arg = args.args.iter().next().unwrap();
        let bits = arg.into_token_stream();
        let res = quote! { eir::frontend::DataType::#id(#bits) };
        return Ok(res.into());
      }
      _ => unreachable!(),
    }
  }
  Err(syn::Error::new(
    raw.span(),
    format!(
      "Expected {{int/uint}}<{{bits}}>, but got {}",
      raw.into_token_stream()
    ),
  ))
}
