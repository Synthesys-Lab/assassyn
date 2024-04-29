use codegen::{emit_body, emit_ports, emit_rets};
use proc_macro::TokenStream;
use quote::quote;
use syn::bracketed;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Token};

mod ast;
mod codegen;
mod parser;

use ast::node;

struct ModuleParser {
  module_name: syn::Ident,
  builder_name: syn::Ident,
  ports: Punctuated<node::PortDecl, Token![,]>,
  parameters: Punctuated<syn::Ident, Token![,]>,
  body: node::Body,
  exposes: Option<Punctuated<syn::Ident, Token![,]>>,
}

impl Parse for ModuleParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tok = input
      .parse::<syn::Ident>()
      .map_err(|e| syn::Error::new(e.span(), "Expected module name"))?;
    let module_name = tok.clone();
    let builder_name = syn::Ident::new(&format!("{}_builder", module_name), tok.span());
    let raw_ports;
    bracketed!(raw_ports in input);
    let ports = raw_ports.parse_terminated(node::PortDecl::parse, Token![,])?;
    let raw_params;
    bracketed!(raw_params in input);
    let params = raw_params.parse_terminated(syn::Ident::parse, Token![,])?;
    let body = input.parse::<node::Body>()?;
    // .expose(<var-id>) is optional
    let exposes = if input.peek(Token![.]) {
      input.parse::<Token![.]>()?;
      let expose_kw = input.parse::<syn::Ident>()?;
      assert_eq!(expose_kw.to_string(), "expose");
      let exposes;
      bracketed!(exposes in input);
      let exposes = exposes.parse_terminated(syn::Ident::parse, Token![,])?;
      Some(exposes)
    } else {
      None
    };

    Ok(ModuleParser {
      module_name,
      builder_name,
      ports,
      parameters: params,
      body,
      exposes,
    })
  }
}

/// Parse a module builder macro.
/// <id> [ <args> ] [ <parameterizables> ] {
///    <body>
/// }
#[proc_macro]
pub fn module_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let parsed_module = parse_macro_input!(input as ModuleParser);

  let module_name = parsed_module.module_name;
  let builder_name = parsed_module.builder_name;

  // codegen ports
  let (port_ids, port_decls, port_peeks) = match emit_ports(&parsed_module.ports) {
    Ok(x) => x,
    Err(e) => return e.to_compile_error().into(),
  };

  let body = match emit_body(&parsed_module.body) {
    Ok(x) => x,
    Err(e) => return e.to_compile_error().into(),
  };

  // Generating parameterized argument list.
  let parameterization: proc_macro2::TokenStream = {
    let parameters = &parsed_module.parameters;
    let mut res = TokenStream::new();
    for elem in parameters.iter() {
      res.extend::<TokenStream>(quote! { #elem: eir::ir::node::BaseNode, }.into());
    }
    res.into()
  };

  let (ret_tys, ret_vals) = emit_rets(&parsed_module.exposes);

  let parameterizable = parsed_module.parameters;
  let res = quote! {
    fn #builder_name (sys: &mut eir::builder::SysBuilder, #parameterization) -> #ret_tys {
      use eir::ir::node::IsElement;
      let module = {
        let res = sys.create_module(stringify!(#module_name), vec![#port_decls]);
        let mut module_mut = res.as_mut::<eir::ir::Module>(sys).expect("[CG] No module found!");
        let raw_ptr = #builder_name as *const ();
        module_mut.set_builder_func_ptr(raw_ptr as usize);
        module_mut.set_parameterizable(vec![#parameterizable]);
        res
      };
      sys.set_current_module(module.clone());
      let ( #port_ids ) = {
        let module = module
          .as_ref::<eir::ir::Module>(&sys)
          .expect("[Init Port] No current module!");
        #port_peeks
        ( #port_ids )
      };
      #body
      #ret_vals
    }
  };

  res.into()
}
