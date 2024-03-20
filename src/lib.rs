use proc_macro::TokenStream;
use quote::quote;
use syn::bracketed;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Token};

mod codegen;
mod parser;

use parser::*;

struct ModuleParser {
  module_name: syn::Ident,
  builder_name: syn::Ident,
  ports: Punctuated<Argument, Token![,]>,
  ext_interf: Punctuated<syn::Ident, Token![,]>,
  body: BodyParser,
}

pub(crate) enum Instruction {
  Assign((syn::Ident, syn::Expr)),
  ArrayAssign((syn::Ident, syn::Expr, syn::Expr)),
}

impl Parse for ModuleParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tok = input
      .parse::<syn::Ident>()
      .map_err(|e| syn::Error::new(e.span(), "Expected module name"))?;
    let module_name = tok.clone();
    let builder_name = syn::Ident::new(&format!("{}_builder", module_name.to_string()), tok.span());
    let raw_ports;
    bracketed!(raw_ports in input);
    let ports = raw_ports.parse_terminated(Argument::parse, Token![,])?;
    let raw_ext_interf;
    bracketed!(raw_ext_interf in input);
    let ext_interf = raw_ext_interf.parse_terminated(syn::Ident::parse, Token![,])?;
    let body = input.parse::<BodyParser>()?;

    let res = Ok(ModuleParser {
      module_name,
      builder_name,
      ports,
      ext_interf,
      body,
    });

    res
  }
}

/// Parse a module builder macro.
/// <id> [ <args> ] [ <external-interfaces> ] {
///    <body>
/// }
#[proc_macro]
pub fn module_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let parsed_module = parse_macro_input!(input as ModuleParser);

  let module_name = parsed_module.module_name;
  let builder_name = parsed_module.builder_name;

  // codegen ports
  let (port_ids, port_decls, port_peeks): (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
  ) = {
    let ports = &parsed_module.ports;
    let mut port_ids = TokenStream::new();
    let mut port_decls = TokenStream::new();
    let mut port_peeks = TokenStream::new();
    for (i, elem) in ports.iter().enumerate() {
      let (id, ty) = (elem.id.clone(), elem.ty.clone());
      port_ids.extend::<TokenStream>(quote! { #id, }.into());
      port_peeks
        .extend::<TokenStream>(quote! { let #id = module.get_input(#i).unwrap().clone(); }.into());
      let ty = proc_macro2::TokenStream::from(ty.clone());
      port_decls
        .extend::<TokenStream>(quote! {eir::frontend::PortInfo::new(stringify!(#id), #ty),}.into());
    }
    (port_ids.into(), port_decls.into(), port_peeks.into())
  };

  let mut body = TokenStream::new();
  for stmt in parsed_module.body.stmts.iter() {
    match codegen::emit_parse_instruction(stmt) {
      Ok(x) => body.extend::<TokenStream>(x),
      Err(e) => return e.to_compile_error().into(),
    }
  }
  let body: proc_macro2::TokenStream = body.into();

  // codegen external interfaces
  let ext_interf: proc_macro2::TokenStream = {
    let ext_interf = &parsed_module.ext_interf;
    let mut res = TokenStream::new();
    for elem in ext_interf.iter() {
      res.extend::<TokenStream>(quote! { #elem, }.into());
    }
    res.into()
  };

  let res = quote! {
    fn #builder_name (sys: &mut eir::frontend::SysBuilder, #ext_interf) -> eir::frontend::BaseNode {
      let module = sys.create_module(stringify!(#module_name), vec![#port_decls]);
      let ( #port_ids ) = {
        let module = module.as_ref::<eir::frontend::Module>(&sys).unwrap();
        #port_peeks
        ( #port_ids )
      };
      #body
      module
    }
  };

  // eprintln!("{}", res);

  res.into()
}
