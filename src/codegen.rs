use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned};

use crate::ast::{
  expr::{DType, Expr},
  node::{ArrayAccess, FuncArgs, Instruction},
};

use eir::frontend::DataType;

pub(crate) fn emit_type(dtype: &DType) -> syn::Result<TokenStream> {
  match &dtype.dtype {
    DataType::Int(bits) => Ok(quote! { eir::frontend::DataType::int(#bits) }.into()),
    DataType::UInt(bits) => Ok(quote! { eir::frontend::DataType::uint(#bits) }.into()),
    DataType::Module(args) => {
      let args = args
        .iter()
        .map(|x| {
          emit_type(&DType {
            span: dtype.span.clone(),
            dtype: x.as_ref().clone(),
          })
        })
        .collect::<Result<Vec<_>, _>>()?;
      let args = args
        .into_iter()
        .map(|x| x.into())
        .collect::<Vec<proc_macro2::TokenStream>>();
      Ok(quote! { eir::frontend::DataType::module(vec![#(#args.into()),*]) }.into())
    }
    _ => Err(syn::Error::new(dtype.span.into(), "Unsupported type")),
  }
}

// TODO(@were): Fully deprecate this later.
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
        let dtype = input.parse::<DType>()?;
        emit_type(&dtype)?
      } else {
        quote! { eir::frontend::DataType::int(32) }.into()
      };
      let ty: proc_macro2::TokenStream = ty.into();
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

pub(crate) fn emit_expr_body(expr: &syn::Expr) -> syn::Result<TokenStream> {
  match expr {
    syn::Expr::MethodCall(method) => {
      let receiver = method.receiver.clone();
      match method.method.to_string().as_str() {
        "add" | "mul" | "sub" | "bitwise_and" | "bitwise_or" | "ilt" => {
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
              "[CG.BinOP] Like \"a.add(b)\" should have only 1 operand in the argument list",
            ));
          }
          Ok(
            quote! {{
              let lhs = #a.clone();
              let rhs = #b.clone();
              let res = sys.#method_id(None, lhs, rhs);
              res
            }}
            .into(),
          )
        }
        "flip" => {
          let method_id = format!("create_{}", method.method.to_string());
          let method_id = syn::Ident::new(&method_id, method.method.span());
          let mut operands = method.args.iter();
          let a = &method.receiver;
          if !operands.next().is_none() {
            return Err(syn::Error::new(
              method.span(),
              "[CG.Unary] Like \"a.flip()\" should have no operand in the argument list",
            ));
          }
          Ok(
            quote! {{
              let res = sys.#method_id(#a.clone());
              res
            }}
            .into(),
          )
        }
        "pop" => {
          let method_id = syn::Ident::new("create_fifo_pop", method.method.span());
          Ok(quote!(sys.#method_id(#receiver.clone(), None);).into())
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

fn emit_parsed_expr(expr: &Expr) -> syn::Result<TokenStream> {
  match expr {
    Expr::Ident(id) => Ok(id.into_token_stream().into()),
    Expr::Const((ty, lit)) => {
      let ty = emit_type(ty)?;
      let ty: proc_macro2::TokenStream = ty.into();
      let res = quote! { sys.get_const_int(#ty, #lit) };
      Ok(res.into())
    }
  }
}

fn emit_array_access(aa: &ArrayAccess) -> syn::Result<proc_macro2::TokenStream> {
  let id = aa.id.clone();
  let idx: proc_macro2::TokenStream = emit_parsed_expr(&aa.idx)?.into();
  Ok(
    quote! {{
      let idx = #idx.clone();
      sys.create_array_ptr(#id.clone(), idx)
    }}
    .into(),
  )
}

pub(crate) fn emit_args(
  func: &syn::Ident,
  args: &FuncArgs,
  eager: bool,
) -> proc_macro2::TokenStream {
  let bind = match args {
    FuncArgs::Bound(binds) => binds
      .iter()
      .map(|(k, v)| {
        let value = emit_parsed_expr(v).expect(format!("Failed to emit {}", quote! {v}).as_str());
        let value: proc_macro2::TokenStream = value.into();
        quote! {
          let value = #value.clone();
          let bind = sys.add_bind(bind, stringify!(#k).to_string(), value, #eager);
        }
      })
      .collect::<Vec<proc_macro2::TokenStream>>(),
    FuncArgs::Plain(vec) => vec
      .iter()
      .map(|x| {
        let value = emit_parsed_expr(x).expect(format!("Failed to emit {}", quote! {x}).as_str());
        let value: proc_macro2::TokenStream = value.into();
        quote! {
          let value = #value.clone();
          let bind = sys.push_bind(bind, value, #eager);
        }
      })
      .collect::<Vec<proc_macro2::TokenStream>>(),
  };
  quote! {
    let bind = sys.get_init_bind(#func.clone());
    #(#bind);*;
  }
  .into()
}

pub(crate) fn emit_parse_instruction(inst: &Instruction) -> syn::Result<TokenStream> {
  Ok(
    match inst {
      Instruction::Assign((left, right)) => {
        let right: proc_macro2::TokenStream = emit_expr_body(right)?.into();
        quote! {
          let #left = #right;
        }
      }
      Instruction::ArrayAssign((aa, right)) => {
        let right: proc_macro2::TokenStream = emit_expr_body(right)?.into();
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
      Instruction::AsyncCall(call) => {
        let func = &call.func;
        let args = &call.args;
        if func.to_string() == "self" {
          quote! {{
            let module = sys
              .get_current_module()
              .expect("[Push Bind] No current module to self.trigger")
              .upcast();
            sys.create_self_trigger();
          }}
        } else {
          let args = emit_args(func, args, false);
          quote! {{
            #args;
            sys.create_trigger_bound(bind);
          }}
        }
      }
      Instruction::Bind((id, call, eager)) => {
        let func = &call.func;
        let args = &call.args;
        let args = emit_args(func, args, *eager);
        quote!(
          let #id = {
            #args;
            bind
          };
        )
      }
      Instruction::SpinCall((lock, call)) => {
        let func = &call.func;
        let args = emit_args(func, &call.args, false);
        let emitted_lock = emit_array_access(lock)?;
        quote! {{
          #args
          let lock = #emitted_lock;
          sys.create_spin_trigger_bound(lock, bind);
        }}
        .into()
      }
      Instruction::ArrayAlloc((id, ty, size)) => {
        let ty = emit_type(ty)?;
        let ty: proc_macro2::TokenStream = ty.into();
        quote! {
          let #id = sys.create_array(#ty, stringify!(#id), #size);
        }
      }
      Instruction::When((cond, body)) => {
        let body = body
          .stmts
          .iter()
          .map(|x| emit_parse_instruction(x))
          .collect::<Vec<_>>();
        let mut unwraped_body: Vec<proc_macro2::TokenStream> = vec![];
        for elem in body.into_iter() {
          match elem {
            Ok(x) => unwraped_body.push(x.into()),
            Err(e) => return Err(e.clone()),
          }
        }
        quote! {{
          let cond = #cond.clone();
          let block = sys.create_block(Some(cond));
          sys.set_current_block(block.clone());
          #(#unwraped_body)*;
          let cur_module = sys
            .get_current_module()
            .expect("[When] No current module")
            .upcast();
          let ip = sys.get_current_ip();
          let ip = ip.next(sys).expect("[When] No next ip");
          sys.set_current_ip(ip);
        }}
      }
    }
    .into(),
  )
}
