use codegen::{emit_attrs, emit_module_body, emit_ports, emit_rets};
use proc_macro::TokenStream;
use quote::quote;
use syn::parenthesized;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Token};

mod ast;
mod codegen;
mod parser;
mod utils;

use ast::node::{self, ParaDecl};

use crate::ast::expr::ModuleAttrs;
use ast::expr::PType;

struct ModuleParser {
  module_name: syn::Ident,
  ports: Punctuated<node::PortDecl, Token![,]>,
  parameters: Punctuated<node::ParaDecl, Token![,]>,
  attrs: Punctuated<syn::Ident, Token![,]>,
  body: node::Body,
  exposes: Option<Punctuated<syn::Ident, Token![,]>>,
}

impl Parse for ModuleParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tok = input
      .parse::<syn::Ident>()
      .map_err(|e| syn::Error::new(e.span(), "Expected module name"))?;
    let module_name = tok.clone();
    // parse the parameters
    let raw_params;
    parenthesized!(raw_params in input);
    let parameters = raw_params.parse_terminated(node::ParaDecl::parse, Token![,])?;
    // parse the ports
    let raw_ports;
    parenthesized!(raw_ports in input);
    let ports = raw_ports.parse_terminated(node::PortDecl::parse, Token![,])?;
    // parse the attributes
    let attrs = input.parse::<ModuleAttrs>()?.attrs;
    // parse the body
    let body = input.parse::<node::Body>()?;
    // .expose(<var-id>) is optional
    let exposes = if input.peek(Token![.]) {
      input.parse::<Token![.]>()?;
      let expose_kw = input.parse::<syn::Ident>()?;
      assert_eq!(expose_kw.to_string(), "expose");
      let exposes;
      parenthesized!(exposes in input);
      let exposes = exposes.parse_terminated(syn::Ident::parse, Token![,])?;
      Some(exposes)
    } else {
      None
    };

    Ok(ModuleParser {
      module_name,
      ports,
      parameters,
      attrs,
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
  let builder_name = syn::Ident::new(&format!("{}_builder", module_name), module_name.span());
  let decl_name = syn::Ident::new(&format!("{}_decl", module_name), module_name.span());
  let impl_name = syn::Ident::new(&format!("{}_impl", module_name), module_name.span());

  // codegen ports
  let (port_ids, port_decls, port_peeks, port_pops) =
    match emit_ports(&parsed_module.ports, &parsed_module.attrs) {
      Ok(x) => x,
      Err(e) => return e.to_compile_error().into(),
    };

  let body = match emit_module_body(&parsed_module.body, port_pops) {
    Ok(x) => x,
    Err(e) => return e.to_compile_error().into(),
  };

  // Generating parameterized argument list.
  let parameterization: proc_macro2::TokenStream = {
    let parameters = &parsed_module.parameters;
    let mut res = TokenStream::new();
    for elem in parameters.iter() {
      let elem_id = elem.clone().id;
      res.extend::<TokenStream>(quote! { #elem_id: eir::ir::node::BaseNode, }.into());
    }
    res.into()
  };

  let (ret_tys, ret_vals) = emit_rets(&parsed_module.exposes);

  let raw_params = parsed_module.parameters;
  let parameterizable = raw_params
    .into_iter()
    .map(|para_decl| para_decl.id)
    .collect::<Punctuated<syn::Ident, syn::token::Comma>>();

  let attrs = match emit_attrs(&parsed_module.attrs) {
    Ok(x) => x,
    Err(e) => return e.to_compile_error().into(),
  };

  let res = quote! {


    fn #decl_name (sys: &mut eir::builder::SysBuilder) -> eir::ir::node::BaseNode {
      use eir::ir::data::DataType;
      let res = sys.create_module(stringify!(#module_name), vec![#port_decls]);
      let mut module_mut = res
        .as_mut::<eir::ir::Module>(sys)
        .expect("[CodeGen] No module found!");
      module_mut.set_attrs(std::collections::HashSet::from([#attrs]));
      res
    }

    fn #impl_name (
      sys: &mut eir::builder::SysBuilder,
      module: eir::ir::node::BaseNode,
      #parameterization
    ) -> #ret_tys {
      use eir::ir::data::DataType;
      use eir::ir::node::IsElement;
      {
        let mut module_mut = module
          .as_mut::<eir::ir::Module>(sys)
          .expect("[CodeGen] No module found!");
        let raw_ptr = #builder_name as *const ();
        module_mut.set_builder_func_ptr(raw_ptr as usize);
        module_mut.set_parameterizable(vec![#parameterizable]);
      }
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

    fn #builder_name (sys: &mut eir::builder::SysBuilder, #parameterization) -> #ret_tys {
      let module = #decl_name(sys);
      #impl_name(sys, module, #parameterizable)
    }

  };

  // Open this when the compiler complains about the generated code.
  // eprintln!("{}", res);

  res.into()
}