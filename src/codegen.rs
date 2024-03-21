use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned};

use crate::{ArrayAccess, Instruction};

pub(crate) struct EmitType(pub(crate) TokenStream);

impl Parse for EmitType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tyid = input.parse::<syn::Ident>()?;
    match tyid.to_string().as_str() {
      "int" | "uint" => {
        input.parse::<syn::Token![::]>()?;
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(EmitType(
          quote! { eir::frontend::DataType::#tyid(#bits) }.into(),
        ))
      }
      _ => {
        return Err(syn::Error::new(
          tyid.span(),
          format!("[CG.Type] Unsupported type: {}", tyid.to_string()),
        ));
      }
    }
  }
}

pub(crate) struct EmitIDOrConst(pub(crate) TokenStream);

impl Parse for EmitIDOrConst {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if let Some(_) = input.cursor().ident() {
      let id = input.clone().parse::<syn::Ident>()?;
      Ok(EmitIDOrConst(id.into_token_stream().into()))
    } else if let Some(_) = input.cursor().literal() {
      let lit = input.parse::<syn::LitInt>()?;
      let ty = if input.peek(syn::Token![.]) {
        input.parse::<syn::Token![.]>()?;
        input.parse::<EmitType>()?
      } else {
        EmitType(quote! { eir::frontend::DataType::int(32) }.into())
      };
      let ty: proc_macro2::TokenStream = ty.0.into();
      let res = quote! { sys.get_const_int(#ty, #lit) };
      Ok(EmitIDOrConst(res.into()))
    } else {
      Err(syn::Error::new(
        input.span(),
        "Expected identifier or literal",
      ))
    }
  }
}

pub(crate) fn emit_expr_body(
  expr: &syn::Expr,
  name: Option<&syn::Ident>,
) -> syn::Result<TokenStream> {
  match expr {
    syn::Expr::MethodCall(method) => {
      let receiver = method.receiver.clone();
      // let args = method.args.iter().map(|arg| emit_expr_body(arg).unwrap());
      match method.method.to_string().as_str() {
        "add" | "mul" | "sub" | "bitwise_and" | "bitwise_or" => {
          let method_id = format!("create_{}", method.method.to_string());
          let method_id = syn::Ident::new(&method_id, method.method.span());
          let mut operands = method.args.iter();
          let a = &method.receiver;
          let b = operands.next().unwrap();
          let b = syn::parse::<EmitIDOrConst>(b.into_token_stream().into())?.0;
          let b: proc_macro2::TokenStream = b.into();
          if !operands.next().is_none() {
            return Err(syn::Error::new(
              method.span(),
              "[CG.BinOP] Should be like \"a.add(b)\" should have only 1 operand in the argument list",
            ));
          }
          Ok(
            quote! {{
              let rhs = #b;
              let res = sys.#method_id(None, #a.clone(), rhs);
              res
            }}
            .into(),
          )
        }
        "pop" => {
          let method_id = syn::Ident::new("create_fifo_pop", method.method.span());
          Ok(quote!(sys.#method_id(&#receiver, None);).into())
        }
        _ => Err(syn::Error::new(
          method.span(),
          format!("Not supported method {}", method.method),
        )),
      }
    }
    syn::Expr::Call(call) => {
      let id = syn::parse::<syn::Ident>(call.func.to_token_stream().into())?;
      match id.to_string().as_str() {
        "array" => {
          let mut args = call.args.iter();
          let raw_ty = args.next().unwrap().clone();
          let ty: proc_macro2::TokenStream =
            match syn::parse::<EmitType>(raw_ty.into_token_stream().into()) {
              Ok(ty) => ty.0.into(),
              Err(e) => return Err(e),
            };
          let size = args.next().unwrap();
          let name = name.unwrap();
          Ok(quote!(sys.create_array(#ty, stringify!(#name), #size);).into())
        }
        _ => {
          return Err(syn::Error::new(
            call.span(),
            format!("[CG.FuncCall] Not supported: {}", quote!(#call)),
          ));
        }
      }
    }
    syn::Expr::Path(path) => {
      let id = syn::parse::<syn::Ident>(path.to_token_stream().into())?;
      Ok(quote!(#id.clone()).into())
    }
    _ => {
      return Err(syn::Error::new(
        expr.span(),
        format!("[CG.Expr] Not supported: {}", quote!(#expr)),
      ));
    }
  }
}

fn emit_array_access(aa: &ArrayAccess) -> syn::Result<proc_macro2::TokenStream> {
  let id = aa.id.clone();
  let idx: proc_macro2::TokenStream = aa.idx.clone().into();
  Ok(
    quote! {{
      let idx = #idx;
      sys.create_array_ptr(#id.clone(), idx)
    }}
    .into(),
  )
}

pub(crate) fn emit_parse_instruction(inst: &Instruction) -> syn::Result<TokenStream> {
  Ok(
    match inst {
      Instruction::Assign((left, right)) => {
        let right: proc_macro2::TokenStream = emit_expr_body(right, Some(left))?.into();
        quote! {
          let #left = #right;
        }
      }
      Instruction::ArrayAssign((aa, right)) => {
        let right: proc_macro2::TokenStream = emit_expr_body(right, None)?.into();
        let array_ptr = emit_array_access(aa)?;
        quote! {{
          let ptr = #array_ptr;
          let value = #right;
          sys.create_array_write(ptr, value);
        }}
      }
      Instruction::ArrayRead((id, aa)) => {
        let array_ptr = emit_array_access(aa)?;
        quote! {
          let #id = {
            let ptr = #array_ptr;
            sys.create_array_read(ptr)
          };
        }
      }
    }
    .into(),
  )
}
