use syn::{braced, parenthesized, parse::Parse, punctuated::Punctuated};

use crate::ast::{
  self,
  expr::LValue,
  node::{self, CallKind, FuncArgs, FuncCall, Statement},
  DType, ExprTerm,
};

impl Parse for node::PortDecl {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input
      .parse::<syn::Ident>()
      .map_err(|e| syn::Error::new(e.span(), "Expected a port id"))?;
    let _ = input
      .parse::<syn::Token![:]>()
      .map_err(|e| syn::Error::new(e.span(), "Expected : to specify the type of the port"))?;
    let ty = input.parse::<DType>()?;
    Ok(node::PortDecl { id, ty })
  }
}

impl Parse for node::KVPair {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let key = input.parse::<syn::Ident>()?;
    let _ = input.parse::<syn::Token![:]>()?;
    let value = input.parse::<ExprTerm>()?;
    Ok(node::KVPair { key, value })
  }
}

impl Parse for node::FuncCall {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let func = input.parse::<syn::Ident>()?;
    let args = if input.peek(syn::token::Brace) {
      let content;
      let _ = braced!(content in input);
      let args = content.parse_terminated(node::KVPair::parse, syn::Token![,])?;
      let args = args
        .into_iter()
        .map(|x| (x.key, x.value))
        .collect::<Vec<_>>();
      FuncArgs::Bound(args)
    } else if input.peek(syn::token::Paren) {
      let content;
      let _ = parenthesized!(content in input);
      let args = content.parse_terminated(ExprTerm::parse, syn::Token![,])?;
      FuncArgs::Plain(args.into_iter().collect::<Vec<_>>())
    } else {
      return Err(syn::Error::new(
        input.span(),
        "Expected a function call with arguments",
      ));
    };
    Ok(node::FuncCall { func, args })
  }
}

struct EmptyParen;

impl Parse for EmptyParen {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let empty;
    parenthesized!(empty in input);
    if !empty.is_empty() {
      return Err(syn::Error::new(
        empty.span(),
        "Expected an inline call followed by a pair of empty \"()\"",
      ));
    }
    Ok(EmptyParen)
  }
}

impl Parse for Statement {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(syn::Ident) {
      let tok_lit = input.cursor().ident().unwrap().0.to_string();
      match tok_lit.as_str() {
        "async_call" | "inline" => {
          let kind = input.parse::<syn::Ident>()?; // async_call/inline
          match kind.to_string().as_str() {
            "async_call" => {
              let call = input.parse::<node::FuncCall>()?;
              Ok(node::Statement::Call((CallKind::Async, call)))
            }
            "inline" => {
              let call = input.parse::<node::FuncCall>()?;
              if let FuncArgs::Bound(_) = call.args {
                return Err(syn::Error::new(
                  call.func.span(),
                  "Expected an inline call with plain arguments",
                ));
              }
              input.parse::<EmptyParen>()?;
              Ok(node::Statement::Call((
                CallKind::Inline(Punctuated::new()),
                call,
              )))
            }
            _ => unreachable!(),
          }
        }
        // when <cond> { ... }
        // wait_until <array-ptr> { ... }
        // cycle <lit-int> { ... }
        "when" | "wait_until" | "cycle" => {
          // TODO(@were): To keep it simple, for now, only a ident is allowed.
          input.parse::<syn::Ident>()?; // when
          let pred = match tok_lit.as_str() {
            "when" => node::BodyPred::Condition(input.parse::<syn::Ident>()?),
            "wait_until" => {
              let pred = input.parse::<node::Body>()?;
              assert!(pred.valued);
              node::BodyPred::WaitUntil(Box::new(pred))
            }
            "cycle" => node::BodyPred::Cycle(input.parse::<syn::LitInt>()?),
            _ => unreachable!(),
          };
          let body = input.parse::<node::Body>()?;
          Ok(node::Statement::BodyScope((pred, Box::new(body))))
        }
        // spin <array-ptr> <func-id> { <id>: <expr> }
        "spin" => {
          input.parse::<syn::Ident>()?; // spin
          let lock = input.parse::<node::ArrayAccess>()?;
          let call = input.parse::<node::FuncCall>()?;
          Ok(node::Statement::Call((CallKind::Spin(lock), call)))
        }
        "log" => {
          input.parse::<syn::Ident>()?; // log
          let args;
          parenthesized!(args in input);
          let args = args.parse_terminated(ast::expr::Expr::parse, syn::Token![,])?;
          Ok(node::Statement::Log(args.into_iter().collect::<Vec<_>>()))
        }
        _ => {
          let lval = input.parse::<LValue>()?;
          match lval {
            LValue::IdentList(ids) => {
              // <id>, ... = inline <expr>
              input.parse::<syn::Token![=]>()?; // =
              assert!(input.peek(syn::Ident));
              match input.cursor().ident().unwrap().0.to_string().as_str() {
                "inline" => {
                  input.parse::<syn::Ident>()?; // inline
                  let expr = input.parse::<FuncCall>()?;
                  input.parse::<EmptyParen>()?;
                  Ok(node::Statement::Call((CallKind::Inline(ids), expr)))
                }
                _ => Err(syn::Error::new(
                  input.span(),
                  "Expected an assignment or an expression",
                )),
              }
            }
            LValue::Ident(id) => {
              if input.is_empty() {
                return Ok(node::Statement::ExprTerm(ExprTerm::Ident(id.clone())));
              }
              // <id> = <expr>
              if input.peek(syn::Token![=]) {
                input.parse::<syn::Token![=]>()?;
                // to handle the expression in k = a[0.int::<32>]
                if input.peek(syn::Ident) && input.peek2(syn::token::Bracket) {
                  let aa = input.parse::<node::ArrayAccess>()?;
                  Ok(node::Statement::ArrayRead((id, aa)))
                  // parse special rules of assignment
                } else if let Some((look, _)) = input.cursor().ident() {
                  match look.to_string().as_str() {
                    // <id> = array(<ty>, <size>); array decl
                    "array" => {
                      input.parse::<syn::Ident>()?; // array
                      let args;
                      syn::parenthesized!(args in input);
                      let ty = args.parse::<DType>()?;
                      args.parse::<syn::Token![,]>()?;
                      let size = args.parse::<syn::LitInt>()?;
                      Ok(node::Statement::ArrayAlloc((id, ty, size)))
                    }
                    // <id> = bind <func-id> { <id>: <expr> }; a partial function call
                    "bind" | "eager_bind" => {
                      input.parse::<syn::Ident>()?; // bind
                      let bind = input.parse::<FuncCall>()?;
                      let eager = look.to_string().as_str().eq("eager_bind");
                      Ok(node::Statement::Bind((id, bind, eager)))
                    }
                    _ => {
                      // fall back to normal assignment
                      let assign = input.parse::<ast::expr::Expr>()?;
                      Ok(node::Statement::Assign((id, assign)))
                    }
                  }
                } else {
                  // fall back to normal assignment
                  let assign = input.parse::<ast::expr::Expr>()?;
                  Ok(node::Statement::Assign((id, assign)))
                }
              } else {
                Err(syn::Error::new(
                  input.span(),
                  "Expected an assignment or an expression",
                ))
              }
            }
            LValue::ArrayAccess(aa) => {
              // <id>[<expr>] = <expr>
              input.parse::<syn::Token![=]>()?;
              let right = input.parse::<ast::expr::Expr>()?;
              Ok(node::Statement::ArrayAssign((aa, right)))
            }
          }
        }
      }
    } else {
      Err(syn::Error::new(
        input.span(),
        "Expected an assignment or an expression",
      ))
    }
  }
}

impl Parse for node::Body {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = braced!(content in input);
    let mut stmts = Vec::new();
    let mut valued = false;
    while !content.is_empty() {
      stmts.push(content.parse::<Statement>()?);
      match stmts.last() {
        Some(Statement::BodyScope(_)) => {}
        Some(Statement::ExprTerm(_)) => {
          if content.is_empty() {
            valued = true;
            break;
          }
        }
        _ => {
          content.parse::<syn::Token![;]>()?;
        }
      }
    }
    Ok(node::Body { stmts, valued })
  }
}
