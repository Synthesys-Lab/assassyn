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
    let builder_name = syn::Ident::new(&format!("{}_builder", module_name.to_string()), tok.span());
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

    let res = Ok(ModuleParser {
      module_name,
      builder_name,
      ports,
      parameters: params,
      body,
      exposes,
    });

    res
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
      port_peeks.extend::<TokenStream>(
        quote! {
          let #id = module.get_port(#i).expect(format!("Index {} exceed!", #i).as_str()).clone();
        }
        .into(),
      );
      let ty: proc_macro2::TokenStream = match codegen::emit_type(&ty) {
        Ok(x) => x.into(),
        Err(e) => return e.to_compile_error().into(),
      };
      port_decls
        .extend::<TokenStream>(quote! {eir::builder::PortInfo::new(stringify!(#id), #ty),}.into());
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
  eprintln!("[Parser] node::Body successfully parsed!");

  // codegen parameterizations
  let parameterization: proc_macro2::TokenStream = {
    let parameters = &parsed_module.parameters;
    let mut res = TokenStream::new();
    for elem in parameters.iter() {
      res.extend::<TokenStream>(quote! { #elem: eir::ir::node::BaseNode, }.into());
    }
    res.into()
  };
  eprintln!("[CodeGen] External interaces successfully generated!");

  let (ret_tys, ret_vals): (proc_macro2::TokenStream, proc_macro2::TokenStream) =
    if let Some(exposes) = parsed_module.exposes {
      let mut vals: proc_macro::TokenStream = quote! { module, }.into();
      let mut tys: proc_macro::TokenStream = quote! { eir::ir::node::BaseNode, }.into();
      for elem in exposes.iter() {
        vals.extend::<TokenStream>(quote! { #elem, }.into());
        tys.extend::<TokenStream>(quote! { eir::ir::node::BaseNode, }.into());
      }
      let vals: proc_macro2::TokenStream = vals.into();
      let tys: proc_macro2::TokenStream = tys.into();
      (quote! { ( #tys ) }, quote! { ( #vals ) })
    } else {
      (quote! { eir::ir::node::BaseNode }, quote! { module })
    };

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

  // eprintln!("Raw Source Code:\n{}", res);

  res.into()
}
