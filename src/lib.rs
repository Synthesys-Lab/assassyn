use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::{braced, bracketed};

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

struct ArgumentParser {
  ids: Vec<syn::Ident>,
  tys: Vec<TokenStream>,
}

impl Parse for ArgumentParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = bracketed!(content in input);
    let mut ids = Vec::new();
    let mut tys = Vec::new();
    while !content.is_empty() {
      let id = content
        .parse::<syn::Ident>()
        .map_err(|e| syn::Error::new(e.span(), "Expected a port id"))?;
      ids.push(id);
      let _ = content
        .parse::<syn::Token![:]>()
        .map_err(|e| syn::Error::new(e.span(), "Expected : to specify the type of the port"))?;
      let ty = content.parse::<TypeParser>()?;
      tys.push(ty.ty);
      if content.is_empty() {
        break;
      }
      content
        .parse::<syn::Token![,]>()
        .map_err(|e| syn::Error::new(e.span(), "Each port should be separated by a comma"))?;
    }
    Ok(ArgumentParser { ids, tys })
  }
}

struct ExternalInterfParser {
  ids: Vec<syn::Ident>,
}

impl Parse for ExternalInterfParser {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = bracketed!(content in input);
    let mut ids = Vec::new();
    while !content.is_empty() {
      let id = input
        .parse::<syn::Ident>()
        .map_err(|e| syn::Error::new(e.span(), "Expected external interface id"))?;
      ids.push(id);
      if content.is_empty() {
        break;
      }
      input.parse::<syn::Token![,]>().map_err(|e| {
        syn::Error::new(
          e.span(),
          "Each external interface id should be separated by a comma",
        )
      })?;
    }
    Ok(ExternalInterfParser { ids })
  }
}

struct ModuleParser {
  module_name: syn::Ident,
  builder_name: syn::Ident,
  ports: ArgumentParser,
  ext_interf: ExternalInterfParser,
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
    let ports = input.parse::<ArgumentParser>()?;
    let ext_interf = input.parse::<ExternalInterfParser>()?;
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
    for (i, elem) in ports.ids.iter().enumerate() {
      port_ids.extend::<TokenStream>(quote! { #elem, }.into());
      port_peeks.extend::<TokenStream>(
        quote! { let #elem = module.get_input(#i).unwrap().clone(); }.into(),
      );
    }
    for elem in ports.ids.iter().zip(ports.tys.iter()) {
      let id = elem.0;
      let ty = proc_macro2::TokenStream::from(elem.1.clone());
      port_decls
        .extend::<TokenStream>(quote! {eir::frontend::PortInfo::new(stringify!(#id), #ty),}.into());
    }
    (port_ids.into(), port_decls.into(), port_peeks.into())
  };

  // codegen external interfaces
  let ext_interf: proc_macro2::TokenStream = {
    let ext_interf = &parsed_module.ext_interf;
    let mut res = TokenStream::new();
    for elem in ext_interf.ids.iter() {
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
