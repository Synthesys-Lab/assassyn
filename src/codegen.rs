use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};

use crate::ast::{
  self,
  expr::{self, DType, ExprTerm},
  node::{ArrayAccess, BodyPred, FuncArgs, Instruction},
};

use eir::ir::data::DataType;

pub(crate) fn emit_type(dtype: &DType) -> syn::Result<TokenStream> {
  match &dtype.dtype {
    DataType::Int(bits) => Ok(quote! { eir::ir::data::DataType::int(#bits) }.into()),
    DataType::UInt(bits) => Ok(quote! { eir::ir::data::DataType::uint(#bits) }.into()),
    DataType::Module(args) => {
      let args = args
        .iter()
        .map(|x| {
          emit_type(&DType {
            span: dtype.span,
            dtype: x.as_ref().clone(),
          })
        })
        .collect::<Result<Vec<_>, _>>()?;
      let args = args
        .into_iter()
        .map(|x| x.into())
        .collect::<Vec<proc_macro2::TokenStream>>();
      Ok(quote! { eir::ir::data::DataType::module(vec![#(#args.into()),*]) }.into())
    }
    _ => Err(syn::Error::new(dtype.span, "Unsupported type")),
  }
}

pub(crate) fn emit_expr_body(expr: &ast::expr::Expr) -> syn::Result<proc_macro2::TokenStream> {
  match expr {
    expr::Expr::Binary((a, op, b)) => match op.to_string().as_str() {
      "add" | "mul" | "sub" | "bitwise_and" | "bitwise_or" | "ilt" | "eq" | "igt" => {
        let method_id = format!("create_{}", op);
        let method_id = syn::Ident::new(&method_id, op.span());
        let a: proc_macro2::TokenStream = emit_parsed_expr(a)?.into();
        let b: proc_macro2::TokenStream = emit_parsed_expr(b)?.into();
        Ok(quote! {{
          let lhs = #a.clone();
          let rhs = #b.clone();
          let res = sys.#method_id(None, lhs, rhs);
          res
        }})
      }
      _ => Err(syn::Error::new(
        op.span(),
        format!(
          "{}:{}: Unsupported method in codegen \"{}\"",
          file!(),
          line!(),
          op
        ),
      )),
    },
    expr::Expr::Unary((a, op)) => match op.to_string().as_str() {
      "flip" => {
        let a: proc_macro2::TokenStream = emit_parsed_expr(a)?.into();
        let method_id = syn::Ident::new(&format!("create_{}", op), op.span());
        Ok(quote! {{
          let res = sys.#method_id(#a.clone());
          res
        }})
      }
      "pop" => {
        let method_id = syn::Ident::new("create_fifo_pop", op.span());
        let a: proc_macro2::TokenStream = emit_parsed_expr(a)?.into();
        Ok(quote!(sys.#method_id(#a.clone(), None);))
      }
      _ => Err(syn::Error::new(
        op.span(),
        format!("Not supported method {}", op),
      )),
    },
    expr::Expr::Slice((a, l, r)) => {
      let method_id = syn::Ident::new("create_slice", a.span());
      let a: proc_macro2::TokenStream = emit_parsed_expr(a)?.into();
      let l: proc_macro2::TokenStream = emit_parsed_expr(l)?.into();
      let r: proc_macro2::TokenStream = emit_parsed_expr(r)?.into();
      Ok(quote! {{
        let src = #a.clone();
        let start = #l;
        let end = #r;
        let res = sys.#method_id(None, src, start, end);
        res
      }})
    }
    expr::Expr::Term(term) => {
      let res = emit_parsed_expr(term)?;
      Ok(res.into())
    }
    expr::Expr::Select((default, cases)) => {
      let mut res: proc_macro2::TokenStream = emit_parsed_expr(default)?.into();
      for (cond, value) in cases.iter() {
        let cond: proc_macro2::TokenStream = emit_parsed_expr(cond)?.into();
        let value: proc_macro2::TokenStream = emit_parsed_expr(value)?.into();
        res = quote! {{
          let carry = #res;
          let cond = #cond.clone();
          let value = #value.clone();
          sys.create_select(cond, value, carry)
        }};
      }
      Ok(res)
    }
  }
}

fn emit_parsed_expr(expr: &ExprTerm) -> syn::Result<TokenStream> {
  match expr {
    ExprTerm::Ident(id) => Ok(id.into_token_stream().into()),
    ExprTerm::Const((ty, lit)) => {
      let ty = emit_type(ty)?;
      let ty: proc_macro2::TokenStream = ty.into();
      let res = quote! { sys.get_const_int(#ty, #lit) };
      Ok(res.into())
    }
    ExprTerm::StrLit(lit) => {
      let value = lit.value();
      Ok(quote! { sys.get_str_literal(#value.to_string()) }.into())
    }
  }
}

fn emit_array_access(aa: &ArrayAccess) -> syn::Result<proc_macro2::TokenStream> {
  let id = aa.id.clone();
  let idx: proc_macro2::TokenStream = emit_parsed_expr(&aa.idx)?.into();
  Ok(quote! {{
    let idx = #idx.clone();
    sys.create_array_ptr(#id.clone(), idx)
  }})
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
        let value = emit_parsed_expr(v).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {v}));
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
        let value = emit_parsed_expr(x).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {x}));
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
}

pub(crate) fn emit_parse_instruction(inst: &Instruction) -> syn::Result<TokenStream> {
  let res: proc_macro2::TokenStream = match inst {
    Instruction::Assign((left, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right)?;
      quote! {
        let #left = #right;
      }
    }
    Instruction::ArrayAssign((aa, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right)?;
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
      let args = emit_args(func, args, false);
      quote! {{
        #args;
        sys.create_trigger_bound(bind);
      }}
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
    }
    Instruction::ArrayAlloc((id, ty, size)) => {
      let ty = emit_type(ty)?;
      let ty: proc_macro2::TokenStream = ty.into();
      quote! {
        let #id = sys.create_array(#ty, stringify!(#id), #size);
      }
    }
    Instruction::BodyScope((pred, body)) => {
      let body = body
        .stmts
        .iter()
        .map(emit_parse_instruction)
        .collect::<Vec<_>>();
      let mut unwraped_body: Vec<proc_macro2::TokenStream> = vec![];
      for elem in body.into_iter() {
        match elem {
          Ok(x) => unwraped_body.push(x.into()),
          Err(e) => return Err(e.clone()),
        }
      }

      let block_pred = match pred {
        BodyPred::Condition(cond) => {
          quote! {
            let cond = #cond.clone();
            let block_pred = eir::ir::block::BlockPred::Condition(cond);
          }
        }
        BodyPred::Lock(lock) => {
          let lock_arr_ptr = emit_array_access(lock).unwrap();
          quote! {
            let lock_arr_ptr = #lock_arr_ptr.clone();
            let block_pred = eir::ir::block::BlockPred::WaitUntil(lock_arr_ptr);
          }
        }
        BodyPred::Cycle(cycle) => {
          quote! {
            let cycle = #cycle.clone();
            let block_pred = eir::ir::block::BlockPred::Cycle(cycle);
          }
        }
      };
      quote! {{
        #block_pred;
        let block = sys.create_block(block_pred);
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
    Instruction::Log(args) => {
      let args = args
        .iter()
        .map(emit_expr_body)
        .collect::<Result<Vec<_>, _>>()?;
      let reassign = args
        .iter()
        .enumerate()
        .map(|(i, x)| {
          let id = syn::Ident::new(&format!("_{}", i), Span::call_site());
          quote! { let #id = #x; }
        })
        .collect::<Vec<_>>();
      let ids = (0..args.len())
        .map(|i| syn::Ident::new(&format!("_{}", i), Span::call_site()))
        .collect::<Vec<_>>();
      let fmt = ids[0].clone();
      let rest = ids[1..].to_vec();
      quote! {{
        #(#reassign)*
        sys.create_log(#fmt, vec![#(#rest),*]);
      }}
    }
  };
  Ok(res.into())
}
