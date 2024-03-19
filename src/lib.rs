use proc_macro2::{Delimiter, TokenTree};
use quote::quote;
use syn::spanned::Spanned;

fn expect_ident(
  prefix: &str,
  pp: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
  allow_none: bool,
) -> Result<Option<proc_macro2::Ident>, syn::Error> {
  let tok = pp.next();
  match tok {
    Some(TokenTree::Ident(ident)) => Ok(ident.into()),
    Some(other) => Err(syn::Error::new(
      other.span(),
      format!(
        "{} Expect an identifier, but got {}",
        prefix,
        other.to_string()
      ),
    )),
    None => {
      if allow_none {
        Ok(None)
      } else {
        Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          format!("{} Expect an identifier, but got end of stream", prefix),
        ))
      }
    }
  }
}

fn expect_punct_alone(
  pp: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
  ch: char,
  allow_none: bool,
) -> Result<Option<proc_macro2::Punct>, syn::Error> {
  let tok = pp.next();
  match &tok {
    Some(TokenTree::Punct(punct)) => {
      if punct.as_char() == ch {
        Ok(punct.clone().into())
      } else {
        Err(syn::Error::new(
          tok.span(),
          format!("Expect '{}', but got '{}'", ch, punct),
        ))
      }
    }
    None => {
      if allow_none {
        Ok(None)
      } else {
        Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          format!("Expect '{}', but got end of stream", ch),
        ))
      }
    }
    _ => Err(syn::Error::new(
      tok.span(),
      format!("Expect '{}', but got {:?}", ch, tok),
    )),
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
  match expect_ident("[Parse Type]", pp, false) {
    Ok(Some(raw_id)) => {
      let ty_id = raw_id.to_string();
      if ty_id.eq("int") || ty_id.eq("uint") {
        expect_punct_alone(pp, '<', false).unwrap();
        let bits = pp.next().unwrap();
        expect_punct_alone(pp, '>', false).unwrap();
        return quote! {eir::frontend::DataType::#raw_id(#bits)}.into();
      } else {
        return syn::Error::new(raw_id.span(), format!("Unsupported type: {}", ty_id))
          .to_compile_error()
          .into();
      }
    }
    Err(e) => {
      return e.to_compile_error().into();
    }
    Ok(None) => unreachable!(),
  }
}

fn parse_module(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
  let mut pp = input.into_iter().peekable();

  let (module_name, builder_name) = {
    let tok = expect_ident("[Module Name]", &mut pp, false)?.unwrap();
    let module_name = tok.clone();
    let builder_name = syn::Ident::new(&format!("{}_builder", module_name.to_string()), tok.span());
    (module_name, builder_name)
  };

  // parse ports
  let (port_ids, port_infos, port_fifo_peeks) = {
    let raw_args = pp.next();
    let span = raw_args.span();
    let mut port_infos = proc_macro2::TokenStream::new();
    let mut port_ids = proc_macro2::TokenStream::new();
    let mut port_fifo_peeks = proc_macro2::TokenStream::new();
    let mut idx: usize = 0;
    match raw_args {
      Some(TokenTree::Group(args)) => {
        assert_eq!(args.delimiter(), Delimiter::Bracket);
        let mut iter = args.stream().into_iter().peekable();
        while let Some(id) = expect_ident("[Arguments]", &mut iter, true)? {
          expect_punct_alone(&mut iter, ':', false)?;
          let ty = parse_type(&mut iter);
          let pi: proc_macro2::TokenStream =
            quote! { eir::frontend::PortInfo::new(stringify!(#id), #ty), };
          port_infos.extend(pi);
          let fifo_peek = quote! { let #id = module.get_input(#idx).unwrap().clone(); };
          port_fifo_peeks.extend(fifo_peek);
          port_ids.extend(quote! { #id, });
          idx += 1;
          expect_punct_alone(&mut iter, ',', true)?;
        }
      }
      _ => {
        return Err(syn::Error::new(
          span,
          format!("Expect a group of arguments, but got {:?}", raw_args),
        ))
      }
    };
    port_ids = quote! { (#port_ids) };
    (port_ids, port_infos, port_fifo_peeks)
  };

  let ext_interf = {
    let raw = pp.next();
    let span = raw.span();
    match raw {
      Some(TokenTree::Group(args)) => {
        assert_eq!(args.delimiter(), Delimiter::Bracket);
        let mut ext_args = proc_macro2::TokenStream::new();
        let mut iter = args.stream().into_iter().peekable();
        while let Some(tok) = expect_ident("[Parse Ext Interf]", &mut iter, true)? {
          ext_args.extend(quote! { #tok: eir::frontend::BaseNode, });
          if let None = expect_punct_alone(&mut iter, ',', true)? {
            break;
          }
        }
        ext_args
      }
      _ => {
        return Err(syn::Error::new(
          span,
          format!(
            "Expect a group of external interfaces in [...], but got {:?}",
            raw
          ),
        ))
      }
    }
  };

  Ok(quote!(
    fn #builder_name (sys: &mut eir::frontend::SysBuilder, #ext_interf) -> eir::frontend::BaseNode {
      let module = sys.create_module(stringify!(#module_name), vec![#port_infos]);
      let #port_ids = {
        let module = module.as_ref::<eir::frontend::Module>(&sys).unwrap();
        #port_fifo_peeks
        #port_ids
      };
      module
    }
  )
  .into())
}

/// Parse a module builder macro.
/// <id> [ <args> ] [ <external-interfaces> ] {
///    <body>
/// }
#[proc_macro]
pub fn module_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let tt2 = proc_macro2::TokenStream::from(input);
  match parse_module(tt2) {
    Ok(tt) => tt.into(),
    Err(e) => e.to_compile_error().into(),
  }
}
