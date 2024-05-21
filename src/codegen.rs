use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

use crate::{
  ast::{
    self,
    expr::{self, DType, ExprTerm},
    node::{ArrayAccess, BodyPred, CallKind, FuncArgs, PortDecl, Statement, WeakSpanned},
  },
  utils::punctuated_span,
};

use eir::{
  backend::simulator::camelize,
  ir::{data::DataType, Opcode},
};

pub(crate) fn emit_type(dtype: &DType) -> syn::Result<proc_macro2::TokenStream> {
  match &dtype.dtype {
    DataType::Int(bits) => Ok(quote_spanned! { dtype.span => DataType::int_ty(#bits) }),
    DataType::UInt(bits) => Ok(quote_spanned! { dtype.span => DataType::uint_ty(#bits) }),
    DataType::Bits(bits) => Ok(quote_spanned! { dtype.span => DataType::raw_ty(#bits) }),
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
      Ok(quote_spanned! { dtype.span => DataType::module(vec![#(#args.into()),*]) })
    }
    _ => Err(syn::Error::new(dtype.span, "Unsupported type")),
  }
}

// TODO(@were): Double confirm this assumption: All these are right values.
pub(crate) fn emit_expr_body(expr: &ast::expr::Expr) -> syn::Result<proc_macro2::TokenStream> {
  match expr {
    expr::Expr::MethodCall((self_, op, operands)) => {
      let opcode = Opcode::from_str(&op.to_string()).unwrap();
      match opcode {
        Opcode::FIFOPop | Opcode::FIFOField { .. } => {
          let method_id = syn::Ident::new(&format!("create_fifo_{}", op), op.span());
          match self_.as_ref() {
            expr::Expr::Term(ExprTerm::Ident(id)) => Ok(quote! {{ sys.#method_id(#id) }}),
            _ => Err(syn::Error::new(
              op.span(),
              "Expected an identifier for valid/peek!",
            )),
          }
        }
        Opcode::Slice => {
          // TODO(@were): Fix the span.
          let method_id = syn::Ident::new("create_slice", Span::call_site());
          let a = emit_expr_body(self_)?;
          let l = emit_expr_body(&operands[0])?;
          let r = emit_expr_body(&operands[1])?;
          Ok(quote! {{
            let src = #a.clone();
            let start = #l;
            let end = #r;
            let res = sys.#method_id(src, start, end);
            res
          }})
        }
        _ => {
          let a = emit_expr_body(self_)?;
          let method_id = syn::Ident::new(format!("create_{}", op).as_str(), op.span());
          let (b_def, b_use) =
            if opcode.arity().unwrap() == 2 || matches!(opcode, Opcode::Cast { .. }) {
              let b = emit_expr_body(&operands[0])?;
              (Some(quote! { let rhs = #b.clone(); }), Some(quote! { rhs }))
            } else if opcode.arity().unwrap() == 1 {
              (None, None)
            } else {
              return Err(syn::Error::new(
                op.span(),
                format!(
                  "Unsupported operator: \"{}\" with arity {}",
                  op,
                  opcode.arity().unwrap_or(0)
                ),
              ));
            };
          Ok(quote_spanned! { op.span() => {
            let src = #a.clone();
            #b_def
            let res = sys.#method_id(src, #b_use);
            res
          }})
        }
      }
    }
    expr::Expr::Term(term) => {
      let res = emit_expr_term(term)?;
      if let ExprTerm::ArrayAccess(_) = term {
        Ok(quote_spanned! { term.span() => {
          let ptr = { #res };
          sys.create_array_read(ptr)
        }})
      } else {
        Ok(res)
      }
    }
    expr::Expr::Select((default, cases)) => {
      let mut res = emit_expr_body(default.as_ref())?;
      for (cond, value) in cases.iter() {
        let cond = emit_expr_body(cond)?;
        let value = emit_expr_body(value)?;
        // TODO(@were): Support span here.
        res = quote! {{
          let carry = { #res }.clone();
          let cond = { #cond }.clone();
          let value = { #value }.clone();
          sys.create_select(cond, value, carry)
        }};
      }
      Ok(res)
    }
    //(DType, syn::LitInt, Option<Punctuated<ExprTerm, Token![,]>>)
    expr::Expr::ArrayAlloc((ty, size, init)) => {
      let ty = emit_type(ty)?;
      let (init_values, init_list) = if let Some(init) = init {
        let mut init_values = Punctuated::new();
        let mut init_list = Punctuated::new();
        for (i, elem) in init.iter().enumerate() {
          let id = syn::Ident::new(&format!("_{}", i), elem.span());
          let value = emit_expr_term(elem)?;
          init_list.push_value(id.clone());
          init_list.push_punct(Token![,](elem.span()));
          init_values.push_value(quote! { let #id = #value });
          init_values.push_punct(Token![;](elem.span()));
        }
        let init_span = punctuated_span(init).unwrap_or(ty.span());
        (
          Some(init_values),
          quote_spanned! { init_span => Some(vec![#init_list]) },
        )
      } else {
        (None, quote_spanned! { ty.span() => None })
      };
      Ok(quote_spanned! {
        ty.span() => {
          #init_values
          sys.create_array(#ty, "array", #size, #init_list)
        }
      })
    }
    expr::Expr::Bind(call) => {
      let func = &call.func;
      let args = &call.args;
      let args = emit_arg_binds(func, args, true);
      Ok(quote! {{ #args; bind }})
    }
  }
}

fn emit_expr_term(expr: &ExprTerm) -> syn::Result<proc_macro2::TokenStream> {
  match expr {
    ExprTerm::Ident(id) => Ok(id.into_token_stream()),
    ExprTerm::Const((ty, lit)) => {
      let ty = emit_type(ty)?;
      let res = quote_spanned! { expr.span() => sys.get_const_int(#ty, #lit) };
      Ok(res)
    }
    ExprTerm::StrLit(lit) => {
      let value = lit.value();
      Ok(quote_spanned! { expr.span() => sys.get_str_literal(#value.to_string()) })
    }
    ExprTerm::ArrayAccess(aa) => emit_array_access(aa),
    ExprTerm::DType(ty) => emit_type(ty),
  }
}

fn emit_array_access(aa: &ArrayAccess) -> syn::Result<proc_macro2::TokenStream> {
  let id = aa.id.clone();
  let idx = emit_expr_body(aa.idx.as_ref())?;
  // TODO(@were): Better span handling later.
  Ok(quote_spanned! { aa.id.span() => {
    let idx = { #idx }.clone();
    sys.create_array_ptr(#id.clone(), idx)
  }})
}

// TODO(@were): Union the func and arg span to have the span.
pub(crate) fn emit_arg_binds(
  func: &syn::Ident,
  args: &FuncArgs,
  is_bind: bool,
) -> proc_macro2::TokenStream {
  // If it is a bind, give a None to respect the callee's eager.
  let override_eager = if is_bind {
    quote! { None }
  } else {
    // If it is a function call, give it a false to anyways disable the callee's eager.
    quote! { Some(false) }
  };
  let bind = match args {
    FuncArgs::Bound(binds) => binds
      .iter()
      .map(|(k, v)| {
        let value = emit_expr_body(v).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {v}));
        quote! {
          let value = { #value }.clone();
          let bind = sys.add_bind(bind, stringify!(#k).to_string(), value, #override_eager);
        }
      })
      .collect::<Vec<proc_macro2::TokenStream>>(),
    FuncArgs::Plain(vec) => vec
      .iter()
      .map(|x| {
        let value = emit_expr_body(x).unwrap_or_else(|_| panic!("Failed to emit {}", quote! {x}));
        quote! {
          let value = { #value }.clone();
          let bind = sys.push_bind(bind, value, #override_eager);
        }
      })
      .collect::<Vec<proc_macro2::TokenStream>>(),
  };
  quote! {
    let bind = sys.get_init_bind(#func.clone());
    #(#bind);*;
  }
}

// FIXME(@were): Quote the whole span of the whole statement.
pub(crate) fn emit_parsed_instruction(inst: &Statement) -> syn::Result<TokenStream> {
  let res: proc_macro2::TokenStream = match inst {
    Statement::Assign((left, right)) => match left {
      expr::LValue::Ident(id) => {
        let right = emit_expr_body(right)?;
        quote_spanned! {
          id.span() =>
            let temp = #right;
            if let Ok(mut expr_mut) = temp.as_mut::<eir::ir::Expr>(sys) {
              expr_mut.set_name(stringify!(#id).to_string());
            }
            if let Ok(mut array_mut) = temp.as_mut::<eir::ir::Array>(sys) {
              array_mut.set_name(stringify!(#id).to_string());
            }
            let #id = temp;
        }
      }
      expr::LValue::ArrayAccess(aa) => {
        let array_ptr = emit_array_access(aa)?;
        let right = emit_expr_body(right)?;
        quote! {{
          let ptr = #array_ptr;
          let value = #right;
          sys.create_array_write(ptr, value);
        }}
      }
      expr::LValue::IdentList(l) => {
        return Err(syn::Error::new(
          l.span(),
          "Assigning to a list of identifiers is not supported",
        ))
      }
    },
    Statement::Call((kind, call)) => match kind {
      CallKind::Async => {
        let args = emit_arg_binds(&call.func, &call.args, false);
        quote! {{
          #args;
          sys.create_async_call(bind);
        }}
      }
      CallKind::Inline(lval) => match &call.args {
        FuncArgs::Plain(args) => {
          let impl_id = syn::Ident::new(&format!("{}_impl", call.func), call.func.span());
          let mut emit_args: Punctuated<proc_macro2::TokenStream, Token![;]> = Punctuated::new();
          let mut arg_ids: Punctuated<syn::Ident, Token![,]> = Punctuated::new();
          for (i, elem) in args.iter().enumerate() {
            let elem = emit_expr_body(elem)?;
            let id = syn::Ident::new(&format!("_{}", i), elem.span());
            emit_args.push(quote! { let #id = { #elem } });
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
    Statement::BodyScope((pred, body)) => {
      let unwraped_body = emit_body(body)?;
      let block_init = match pred {
        BodyPred::Condition(ref cond) => quote! {
          let cond = #cond.clone();
          let block_pred = eir::ir::block::BlockKind::Condition(cond);
          let block = sys.create_block(block_pred);
        },
        BodyPred::Cycle(cycle) => quote! {
          let cycle = #cycle.clone();
          let block_pred = eir::ir::block::BlockKind::Cycle(cycle);
          let block = sys.create_block(block_pred);
        },
        _ => panic!("wait_until should only be the root of a module"),
      };
      let emission = quote! {
        #block_init;
        sys.set_current_block(block.clone());
        #unwraped_body
        let ip = sys.get_current_ip();
        let ip = ip.next(sys).expect("[When] No next ip");
        sys.set_current_ip(ip);
      };
      quote! {{
        #emission
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

fn block_value_id(body: &ast::node::Body) -> syn::Ident {
  let ptr = body as *const _ as usize;
  syn::Ident::new(&format!("valued_of_{}", ptr), Span::call_site())
}

pub(crate) fn emit_module_body(
  body: &ast::node::Body,
  implicit_pops: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
  if body.stmts.len() == 1 {
    if let Statement::BodyScope((BodyPred::WaitUntil(lock), body)) = &body.stmts[0] {
      let lock_emission = emit_body(lock).unwrap();
      let value_id = block_value_id(lock);
      assert!(lock.valued);
      let unwraped_body = emit_body(body)?;
      return Ok(quote! {
        sys.set_current_block_wait_until();
        let block_restored = sys.get_current_block().unwrap().upcast();
        let valued_block = {
          let master = sys.get_current_block().unwrap().upcast();
          let master = master.as_ref::<eir::ir::block::Block>(sys).unwrap();
          match master.get_kind() {
            eir::ir::block::BlockKind::WaitUntil(valued_block) => valued_block.clone(),
            _ => unreachable!(),
          }
        };
        sys.set_current_block(valued_block.clone());
        #lock_emission
        valued_block.as_mut::<eir::ir::block::Block>(sys).unwrap().set_value(#value_id);
        sys.set_current_block(block_restored);
        #implicit_pops
        #unwraped_body
      });
    }
  }
  let body = emit_body(body)?;
  Ok(quote! {
    #implicit_pops
    #body
  })
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
    let value_id = block_value_id(body);
    return Ok(quote! {
      #res;
      let #value_id = #value;
    });
  }
  Ok(res.into())
}

pub(crate) fn emit_ports(
  ports: &Punctuated<PortDecl, Token![,]>,
  module_attrs: &Punctuated<syn::Ident, Token![,]>,
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
  let explicit_pop = module_attrs.iter().any(|x| x.eq("explicit_pop"));
  for (i, elem) in ports.iter().enumerate() {
    let (id, ty) = (elem.id.clone(), elem.ty.clone());
    // IDs: <id>, <id>, ...
    port_ids.push(id.clone());
    port_ids.push_punct(Token![,](id.span()));
    let err_log = syn::LitStr::new(&format!("Index {} exceed!", i), id.span());
    // Peek the port instances
    port_peeks.push(quote! { let #id = module.get_port(#i).expect(#err_log).upcast() });
    port_peeks.push_punct(Token![;](id.span()));
    // Declarations: <id>: <ty>,
    let ty = emit_type(&ty)?;
    port_decls.push(quote! { eir::builder::PortInfo::new(stringify!(#id), #ty) });
    port_decls.push_punct(Token![,](id.span()));
    // Pop the port instances
    if !explicit_pop {
      port_pops.push(quote! { let #id = sys.create_fifo_pop(#id.clone()) });
      port_pops.push_punct(Token![;](id.span()));
    }
  }
  Ok((
    quote! {#port_ids},
    quote! {#port_decls},
    quote! {#port_peeks},
    quote! {#port_pops},
  ))
}

pub(crate) fn emit_attrs(
  attrs: &Punctuated<syn::Ident, Token![,]>,
) -> syn::Result<Punctuated<proc_macro2::TokenStream, Token![,]>> {
  let mut res = Punctuated::new();
  for attr in attrs.iter() {
    let camelized = camelize(&attr.to_string());
    let attr = syn::Ident::new(&camelized, attr.span());
    res.push_value(quote! { eir::ir::module::Attribute::#attr });
    res.push_punct(Token![,](attr.span()));
  }
  Ok(res)
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
