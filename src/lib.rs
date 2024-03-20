use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{braced, bracketed};
use syn::{parse_macro_input, Token};

struct TypeParser {
  ty: TokenStream,
}

impl Parse for TypeParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input.parse::<syn::Ident>()?;
    match id.to_string().as_str() {
      "int" | "uint" => {
        let _ = input
          .parse::<syn::Token![<]>()
          .map_err(|e| syn::Error::new(e.span(), "Expected <"))?;
        let bits = input.parse::<syn::LitInt>()?;
        let _ = input
          .parse::<syn::Token![>]>()
          .map_err(|e| syn::Error::new(e.span(), "Expected >"))?;
        Ok(TypeParser {
          ty: quote! {eir::frontend::DataType::#id(#bits)}.into(),
        })
      }
      _ => Err(syn::Error::new(
        id.span(),
        format!("Unsupported type: {}", id.to_string()),
      )),
    }
  }
}

struct Argument {
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

struct ModuleParser {
  module_name: syn::Ident,
  builder_name: syn::Ident,
  ports: Punctuated<Argument, Token![,]>,
  ext_interf: Punctuated<syn::Ident, Token![,]>,
  body: BodyParser,
}

enum Instruction {
  Assign(syn::Ident, syn::Expr),
}

struct BodyParser {
  stmts: Vec<Instruction>,
}

impl Parse for BodyParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = braced!(content in input);
    let mut stmts = Vec::new();
    while !content.is_empty() {
      let master = content.parse::<syn::Ident>()?;
      // a = <expr>
      if content.peek(syn::Token![=]) {
        content.parse::<syn::Token![=]>()?;
        let expr = content.parse::<syn::Expr>()?;
        content.parse::<syn::Token![;]>()?;
        stmts.push(Instruction::Assign(master, expr));
      }
    }
    Ok(BodyParser { stmts })
  }
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

  eprintln!("module_name: {:?} parsed!", parsed_module.module_name);

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

  // codegen external interfaces
  let ext_interf: proc_macro2::TokenStream = {
    let ext_interf = &parsed_module.ext_interf;
    let mut res = TokenStream::new();
    for elem in ext_interf.iter() {
      res.extend::<TokenStream>(quote! { #elem, }.into());
    }
    res.into()
  };

  eprintln!("codegen is done!");

  let res = quote! {
    fn #builder_name (sys: &mut eir::frontend::SysBuilder, #ext_interf) -> eir::frontend::BaseNode {
      let module = sys.create_module(stringify!(#module_name), vec![#port_decls]);
      let ( #port_ids ) = {
        let module = module.as_ref::<eir::frontend::Module>(&sys).unwrap();
        #port_peeks
        ( #port_ids )
      };
      module
    }
  };

  eprintln!("{}", res);

  res.into()
}
