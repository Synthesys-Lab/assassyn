use proc_macro2::{Delimiter, Punct, Span, TokenTree};
use quote::quote;

fn expect_ident(
  pp: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> proc_macro2::Ident {
  if let Some(tok) = pp.next() {
    match tok {
      TokenTree::Ident(ident) => ident,
      _ => {
        panic!("Expect an identifier, but got {}", tok.to_string());
      }
    }
  } else {
    panic!("Expect an identifier, but got nothing");
  }
}

/// Parse a type.
///
/// Supported types: int<bits>, uint<bits>
///
/// TODO(@were): Support module callback type: module((types,)*)
fn parse_type(
  pp: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> proc_macro2::TokenStream {
  let raw_id = expect_ident(pp);
  let ty_id = raw_id.to_string();
  if ty_id.eq("int") || ty_id.eq("uint") {
    pp.next().unwrap();
    let bits = pp.next().unwrap();
    pp.next().unwrap();
    return quote! {eda4eda::frontend::DataType::#raw_id(#bits)}.into();
  }
  panic!("{}: Unexpected identifier {}", line!(), raw_id.to_string());
}

// TODO(@were): This can be implemented in a iterator for performance improvement.
fn split_token_stream(
  a: proc_macro2::TokenStream,
  id: String,
) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenTree)> {
  let mut vec = vec![];
  let mut current = proc_macro2::TokenStream::new();
  let mut a = a.into_iter().peekable();
  while let Some(tok) = a.next() {
    if tok.to_string() == id {
      vec.push((current, tok));
      current = proc_macro2::TokenStream::new();
    } else {
      current.extend(proc_macro2::TokenStream::from(tok));
    }
  }
  vec.push((
    current,
    proc_macro2::TokenTree::Punct(Punct::new('\0', proc_macro2::Spacing::Alone)),
  ));
  vec
}

/// Parse a module builder macro.
/// <id> [ <args> ] [ <external-interfaces> ] {
///    <body>
/// }
#[proc_macro]
pub fn module_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let tt2 = proc_macro2::TokenStream::from(input);

  let mut pp = tt2.into_iter().peekable();

  let module_name = expect_ident(&mut pp);
  let builder_name = format!("{}_builder", module_name.to_string());
  let builder_name = syn::Ident::new(&builder_name, Span::call_site());

  // parse arguments
  let args = {
    let raw_args = pp.next();
    let mut expanded = proc_macro2::TokenStream::new();
    let raw_args = match raw_args {
      Some(TokenTree::Group(args)) => {
        assert_eq!(args.delimiter(), Delimiter::Bracket);
        split_token_stream(args.stream(), ",".to_string())
      }
      _ => panic!(
        "{}: Expect a group of arguments, but got {:?}",
        line!(),
        raw_args
      ),
    };
    for ts in raw_args.into_iter() {
      let raw_arg = split_token_stream(ts.0, ":".to_string());
      let (ident, ty) = (
        raw_arg.get(0).unwrap().0.clone(),
        raw_arg.get(1).unwrap().0.clone(),
      );
      let ty = parse_type(&mut ty.clone().into_iter().peekable());
      let ident = ident.to_string();
      let pi = quote! {
        eda4eda::frontend::PortInfo::new(#ident, #ty)
      };
      expanded.extend(pi);
      expanded.extend(proc_macro2::TokenStream::from(ts.1));
    }
    expanded
  };

  let ext_interf = {
    let raw = pp.next();
    let mut expanded = proc_macro2::TokenStream::new();
    let stripped = match raw {
      Some(TokenTree::Group(args)) => {
        assert_eq!(args.delimiter(), Delimiter::Bracket);
        split_token_stream(args.stream(), ",".to_string())
      }
      _ => panic!(
        "{}: Expect a group of external interfaces, but got {:?}",
        line!(),
        raw
      ),
    };
    for ts in stripped.into_iter() {
      let (toks, sep) = ts;
      let id = {
        let mut iter = toks.into_iter().peekable();
        let res = iter.next();
        match &res {
          Some(TokenTree::Ident(_)) => {}
          _ => panic!("{}: Expect an identifier, but got {:?}", line!(), res),
        }
        assert!(iter.next().is_none());
        res
      };
      expanded.extend(id);
      expanded.extend(proc_macro2::TokenStream::from(sep));
    }
    expanded
  };

  quote!(
    fn #builder_name (sys: &mut eda4eda::frontend::SysBuilder, #ext_interf) -> eda4eda::BaseNode {
      let module = sys.create_module(stringify!(#module_name), vec![#args]);
      module
    }
  )
  .into()
}
