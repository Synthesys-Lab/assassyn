use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned};

use crate::Instruction;

pub(crate) struct ExprToType(pub(crate) TokenStream);

impl Parse for ExprToType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tyid = input.parse::<syn::Ident>()?;
    match tyid.to_string().as_str() {
      "int" | "uint" => {
        input.parse::<syn::Token![::]>()?;
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(ExprToType(
          quote!(eir::frontend::DataType::#tyid(#bits)).into(),
        ))
      }
      _ => {
        return Err(syn::Error::new(
          tyid.span(),
          format!("Unsupported type: {}", tyid.to_string()),
        ));
      }
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
          if !operands.next().is_none() {
            return Err(syn::Error::new(
              method.span(),
              "Binary op call like \"a.add(b)\" should have only 1 operand in the argument list",
            ));
          }
          Ok(quote!(sys.#method_id(None, &#a, &#b);).into())
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
          let mut sys = syn::Ident::new("sys", call.func.span());
          let raw_ty = args.next().unwrap().clone();
          let ty: proc_macro2::TokenStream =
            match syn::parse::<ExprToType>(raw_ty.into_token_stream().into()) {
              Ok(ty) => ty.0.into(),
              Err(e) => return Err(e),
            };
          let size = args.next().unwrap();
          let name = name.unwrap();
          Ok(quote!(sys.create_array(&#ty, stringify!(#name), #size);).into())
        }
        _ => {
          return Err(syn::Error::new(
            call.span(),
            format!("Not supported func call {}", quote!(#call)),
          ));
        }
      }
    }
    _ => {
      return Err(syn::Error::new(
        expr.span(),
        format!("Not supported expr {}", quote!(#expr)),
      ));
    }
  }
}

pub(crate) fn emit_parse_instruction(inst: &Instruction) -> syn::Result<TokenStream> {
  match inst {
    Instruction::Assign((left, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right, Some(left))?.into();
      Ok(quote! { let #left = #right; }.into())
    }
    Instruction::ArrayAssign((array, idx, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right, None)?.into();
      let idx_id = syn::Ident::new("idx", idx.span());
      let idx: proc_macro2::TokenStream = emit_expr_body(idx, Some(&idx_id))?.into();
      Ok(
        quote! {
          let #idx_id = #idx;
          let ptr = sys.create_array_ptr(&#array, &#idx);
          sys.create_array_write(&ptr, #right);
        }
        .into(),
      )
    }
  }
}
