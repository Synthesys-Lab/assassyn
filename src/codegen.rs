use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::Instruction;

pub(crate) fn expr_to_type(raw: syn::Expr) -> syn::Result<TokenStream> {
  if let syn::Expr::Path(path) = raw {
    assert_eq!(path.path.segments.len(), 1);
    let stripped = path.path.segments.iter().next().unwrap();
    let id = stripped.ident.clone();
    match &stripped.arguments {
      syn::PathArguments::AngleBracketed(args) => {
        assert_eq!(args.args.len(), 1);
        let arg = args.args.iter().next().unwrap();
        let bits = arg.into_token_stream();
        let res = quote! { eir::frontend::DataType::#id(#bits) };
        return Ok(res.into());
      }
      _ => unreachable!(),
    }
  }
  Err(syn::Error::new(
    raw.span(),
    format!(
      "Expected {{int/uint}}<{{bits}}>, but got {}",
      raw.into_token_stream()
    ),
  ))
}

pub(crate) fn emit_expr_body(expr: &syn::Expr, name: &syn::Ident) -> syn::Result<TokenStream> {
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
      let func = call.func.clone();
      match func.as_ref() {
        syn::Expr::Path(path) => {
          let id = path.path.segments.iter().next().unwrap().ident.clone();
          match id.to_string().as_str() {
            "array" => {
              let mut args = call.args.iter();
              let ty: proc_macro2::TokenStream = expr_to_type(args.next().unwrap().clone())?.into();
              let size = args.next().unwrap();
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
      let right: proc_macro2::TokenStream = emit_expr_body(right, left)?.into();
      Ok(
        quote! {
          let #left = #right;
        }
        .into(),
      )
    }
  }
}
