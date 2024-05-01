use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

use crate::ast::{
  self,
  expr::{self, DType, ExprTerm},
  node::{ArrayAccess, BodyPred, CallKind, FuncArgs, PortDecl, Statement},
};

use eir::ir::data::DataType;

pub(crate) fn emit_type(dtype: &DType) -> syn::Result<TokenStream> {
  match &dtype.dtype {
    DataType::Int(bits) => Ok(quote! { eir::ir::data::DataType::int_ty(#bits) }.into()),
    DataType::UInt(bits) => Ok(quote! { eir::ir::data::DataType::uint_ty(#bits) }.into()),
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
        let a: proc_macro2::TokenStream = emit_expr_term(a)?.into();
        let b: proc_macro2::TokenStream = emit_expr_term(b)?.into();
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
        let a: proc_macro2::TokenStream = emit_expr_term(a)?.into();
        let method_id = syn::Ident::new(&format!("create_{}", op), op.span());
        Ok(quote! {{
          let res = sys.#method_id(#a.clone());
          res
        }})
      }
      "pop" => {
        let method_id = syn::Ident::new(&format!("create_fifo_{}", op), op.span());
        let a: proc_macro2::TokenStream = emit_expr_term(a)?.into();
        Ok(quote!(sys.#method_id(#a.clone(), None);))
      }
      "valid" | "peek" => {
        let method_id = syn::Ident::new(&format!("create_fifo_{}", op), op.span());
        // @were: I am not sure  if this is a temporary hack or a long-term solution
        // to get compatible with the current implicit FIFO pop.
        // Before, the ID of the ExprTerm directly refers to the FIFO instance, but now
        // after the implicit FIFO pop, it refers to a value from the FIFO.
        // However, when generating a valid, it will typically be used in the wait_until
        // block before the pop, so we do not worry about "popping before validation".
        //
        // This is kinda back-and-forth in the code generator.
        let fifo_self = match a {
          ExprTerm::Ident(id) => {
            let name = id.to_string();
            quote! {{
              let module = module.as_ref::<eir::ir::Module>(sys).unwrap();
              module.get_port_by_name(#name).unwrap_or_else(|| {
                panic!("Module {} has no port named {}", module.get_name(), #name)
              }).upcast()
            }}
          }
          _ => {
            return Err(syn::Error::new(
              op.span(),
              "Expected an identifier for valid/peek!",
            ))
          }
        };
        Ok(quote! {{
          let fifo = #fifo_self;
          sys.#method_id(fifo)
        }})
      }
      _ => Err(syn::Error::new(
        op.span(),
        format!("Not supported method {}", op),
      )),
    },
    expr::Expr::Slice((a, l, r)) => {
      let method_id = syn::Ident::new("create_slice", a.span());
      let a: proc_macro2::TokenStream = emit_expr_term(a)?.into();
      let l: proc_macro2::TokenStream = emit_expr_term(l)?.into();
      let r: proc_macro2::TokenStream = emit_expr_term(r)?.into();
      Ok(quote! {{
        let src = #a.clone();
        let start = #l;
        let end = #r;
        let res = sys.#method_id(None, src, start, end);
        res
      }})
    }
    expr::Expr::Term(term) => {
      let res = emit_expr_term(term)?;
      Ok(res.into())
    }
    expr::Expr::Select((default, cases)) => {
      let mut res: proc_macro2::TokenStream = emit_expr_term(default)?.into();
      for (cond, value) in cases.iter() {
        let cond: proc_macro2::TokenStream = emit_expr_term(cond)?.into();
        let value: proc_macro2::TokenStream = emit_expr_term(value)?.into();
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

fn emit_expr_term(expr: &ExprTerm) -> syn::Result<TokenStream> {
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
  let idx: proc_macro2::TokenStream = emit_expr_term(&aa.idx)?.into();
  Ok(quote! {{
    let idx = #idx.clone();
    sys.create_array_ptr(#id.clone(), idx)
  }})
}

pub(crate) fn emit_arg_binds(
  func: &syn::Ident,
  args: &FuncArgs,
  eager: bool,
) -> proc_macro2::TokenStream {
  let bind = match args {
    FuncArgs::Bound(binds) => binds
      .iter()
      .map(|(k, v)| {
        let value = emit_expr_term(v).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {v}));
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
        let value = emit_expr_term(x).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {x}));
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

pub(crate) fn emit_parsed_instruction(inst: &Statement) -> syn::Result<TokenStream> {
  let res: proc_macro2::TokenStream = match inst {
    Statement::Assign((left, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right)?;
      quote! {
        let #left = #right;
      }
    }
    Statement::ArrayAssign((aa, right)) => {
      let right: proc_macro2::TokenStream = emit_expr_body(right)?;
      let array_ptr = emit_array_access(aa)?;
      quote! {{
        let ptr = #array_ptr;
        let value = #right;
        sys.create_array_write(ptr, value);
      }}
    }
    Statement::ArrayRead((id, aa)) => {
      let array_ptr = emit_array_access(aa)?;
      quote! {
        let #id = {
          let ptr = #array_ptr;
          sys.create_array_read(ptr)
        };
      }
    }
    Statement::Bind((id, call, eager)) => {
      let func = &call.func;
      let args = &call.args;
      let args = emit_arg_binds(func, args, *eager);
      quote!(
        let #id = {
          #args;
          bind
        };
      )
    }
    Statement::Call((kind, call)) => match kind {
      CallKind::Spin(lock) => {
        let args = emit_arg_binds(&call.func, &call.args, false);
        let emitted_lock = emit_array_access(lock)?;
        quote! {{
          #args;
          let lock = #emitted_lock;
          sys.create_spin_trigger_bound(lock, bind);
        }}
      }
      CallKind::Async => {
        let args = emit_arg_binds(&call.func, &call.args, false);
        quote! {{
          #args;
          sys.create_trigger_bound(bind);
        }}
      }
      CallKind::Inline(lval) => match &call.args {
        FuncArgs::Plain(args) => {
          let impl_id = syn::Ident::new(&format!("{}_impl", call.func), call.func.span());
          let mut emit_args: Punctuated<proc_macro2::TokenStream, Token![;]> = Punctuated::new();
          let mut arg_ids: Punctuated<syn::Ident, Token![,]> = Punctuated::new();
          for (i, elem) in args.iter().enumerate() {
            let elem: proc_macro2::TokenStream = emit_expr_term(elem)?.into();
            let id = syn::Ident::new(&format!("_{}", i), elem.span());
            emit_args.push(quote! { let #id = #elem });
            emit_args.push_punct(Token![;](elem.span()));
            arg_ids.push(id);
            arg_ids.push_punct(Token![,](elem.span()));
          }
          let lval = if lval.is_empty() {
            quote! {let _ = }
          } else {
            quote! { let (_, #lval) = }
          };
          quote! {
            #lval {
              #emit_args
              #impl_id(sys, module, #arg_ids)
            };
          }
        }
        FuncArgs::Bound(bound) => {
          return Err(syn::Error::new(
            bound.first().map_or(Span::call_site(), |x| x.0.span()),
            "Inline call does not support bound arguments",
          ))
        }
      },
    },
    Statement::ArrayAlloc((id, ty, size)) => {
      let ty = emit_type(ty)?;
      let ty: proc_macro2::TokenStream = ty.into();
      quote! {
        let #id = sys.create_array(#ty, stringify!(#id), #size);
      }
    }
    Statement::BodyScope((pred, body)) => {
      let unwraped_body = emit_body(body)?;

      let block_init = match pred {
        BodyPred::Condition(cond) => {
          quote! {{
            let cond = #cond.clone();
            let block_pred = eir::ir::block::BlockKind::Condition(cond);
            (sys.create_block(block_pred), true)
          }}
        }
        BodyPred::Cycle(cycle) => {
          quote! {{
            let cycle = #cycle.clone();
            let block_pred = eir::ir::block::BlockKind::Cycle(cycle);
            (sys.create_block(block_pred), true)
          }}
        }
        BodyPred::WaitUntil(lock) => {
          let lock_emission = emit_body(lock).unwrap();
          quote! {{
            sys.set_current_block_wait_until();
            let master = sys.get_current_block().unwrap().upcast();
            {
              let master = master.as_ref::<eir::ir::block::Block>(sys).unwrap();
              if let eir::ir::block::BlockKind::WaitUntil(valued_block) = master.get_kind() {
                sys.set_current_block(valued_block.clone());
                let cond_value = #lock_emission;
                let block = sys.get_current_block().unwrap().upcast();
                block.as_mut::<eir::ir::block::Block>(sys).unwrap().set_value(cond_value);
              }
            }
            (master, false)
          }}
        }
      };
      quote! {{
        let (block, tick_ip) = #block_init;
        sys.set_current_block(block.clone());
        #unwraped_body
        let cur_module = sys
          .get_current_module()
          .expect("[When] No current module")
          .upcast();
        if tick_ip {
          let ip = sys.get_current_ip();
          let ip = ip.next(sys).expect("[When] No next ip");
          sys.set_current_ip(ip);
        }
      }}
    }
    Statement::Log(args) => {
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
    Statement::ExprTerm(_) => {
      return Err(syn::Error::new(
        Span::call_site(),
        format!(
          "{}: {}: ExprTerm should not be emitted here",
          file!(),
          line!()
        ),
      ))
    }
  };
  Ok(res.into())
}

/// Emit a braced block of instructions.
pub(crate) fn emit_body(body: &ast::node::Body) -> syn::Result<proc_macro2::TokenStream> {
  let mut res = TokenStream::new();
  let (n, value) = if body.valued {
    let mut value = None;
    assert!(match body.stmts.last() {
      Some(ast::node::Statement::ExprTerm(ExprTerm::Ident(id))) => {
        value = id.clone().into();
        true
      }
      _ => false,
    });
    (body.stmts.len() - 1, value)
  } else {
    (body.stmts.len(), None)
  };
  for stmt in body.stmts.iter().take(n) {
    let segment = emit_parsed_instruction(stmt)?;
    res.extend::<TokenStream>(segment);
  }
  if let Some(value) = value {
    let res: proc_macro2::TokenStream = res.into();
    return Ok(quote! {{
      #res;
      #value
    }});
  }
  Ok(res.into())
}

pub(crate) fn emit_ports(
  ports: &Punctuated<PortDecl, Token![,]>,
) -> syn::Result<(
  proc_macro2::TokenStream,
  proc_macro2::TokenStream,
  proc_macro2::TokenStream,
  proc_macro2::TokenStream,
)> {
  let mut port_ids: Punctuated<syn::Ident, Token![,]> = Punctuated::new();
  let mut port_decls: Punctuated<proc_macro2::TokenStream, Token![,]> = Punctuated::new();
  let mut port_peeks: Punctuated<proc_macro2::TokenStream, Token![;]> = Punctuated::new();
  let mut port_pops: Punctuated<proc_macro2::TokenStream, Token![;]> = Punctuated::new();
  for (i, elem) in ports.iter().enumerate() {
    let (id, ty) = (elem.id.clone(), elem.ty.clone());
    // IDs: <id>, <id>, ...
    port_ids.push(id.clone());
    port_ids.push_punct(Token![,](id.span()));
    let err_log = syn::LitStr::new(&format!("Index {} exceed!", i), id.span());
    // Peek the port instances
    port_peeks.push(quote! { let #id = module.get_port(#i).expect(#err_log).clone() });
    port_peeks.push_punct(Token![;](id.span()));
    // Declarations: <id>: <ty>,
    let ty: proc_macro2::TokenStream = emit_type(&ty)?.into();
    port_decls.push(quote! { eir::builder::PortInfo::new(stringify!(#id), #ty) });
    port_decls.push_punct(Token![,](id.span()));
    // Pop the port instances
    port_pops.push(quote! { let #id = sys.create_fifo_pop(#id.clone(), None) });
    port_pops.push_punct(Token![;](id.span()));
  }
  Ok((
    quote! {#port_ids},
    quote! {#port_decls},
    quote! {#port_peeks},
    quote! {#port_pops},
  ))
}

pub(crate) fn emit_rets(
  x: &Option<Punctuated<syn::Ident, Token![,]>>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
  if let Some(exposes) = x {
    // Types: BaseNode(module), BaseNode, BaseNode, ...
    let mut tys: Punctuated<proc_macro2::TokenStream, Token![,]> = Punctuated::new();
    tys.push(quote! { eir::ir::node::BaseNode });
    tys.push_punct(Token![,](Span::call_site()));
    // Values: module, <id>, <id>, ...
    let mut vals: Punctuated<syn::Ident, Token![,]> = Punctuated::new();
    vals.push(syn::Ident::new("module", Span::call_site()));
    vals.push_punct(Default::default());
    for elem in exposes.iter() {
      // Pushing values
      vals.push(elem.clone());
      vals.push_punct(Token![,](elem.span()));
      // Pushing types
      tys.push(quote! { eir::ir::node::BaseNode });
      tys.push_punct(Token![,](elem.span()));
    }
    (quote! { ( #tys ) }, quote! { ( #vals ) })
  } else {
    (quote! { eir::ir::node::BaseNode }, quote! { module })
  }
}
