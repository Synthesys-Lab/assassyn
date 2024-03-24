use syn::{braced, parenthesized, parse::Parse, Ident};

use crate::ast::{
  node::{self, FuncArgs},
  DType, Expr,
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

impl Parse for node::ArrayAccess {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input.parse::<syn::Ident>()?;
    let idx;
    syn::bracketed!(idx in input);
    let idx = idx.parse::<Expr>()?;
    Ok(node::ArrayAccess { id, idx })
  }
}

impl Parse for node::KVPair {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let key = input.parse::<syn::Ident>()?;
    let _ = input.parse::<syn::Token![:]>()?;
    let value = input.parse::<Expr>()?;
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
      let args = content.parse_terminated(Expr::parse, syn::Token![,])?;
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

impl Parse for node::Body {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = braced!(content in input);
    let mut stmts = Vec::new();
    while !content.is_empty() {
      if content.peek(syn::token::Async) {
        content.parse::<syn::token::Async>()?;
        // async self {}
        if content.peek(syn::token::SelfValue) {
          content.parse::<syn::token::SelfValue>()?;
          let _placeholder;
          braced!(_placeholder in content);
          let func_call = node::FuncCall {
            func: Ident::new("self", content.span()),
            args: FuncArgs::Plain(vec![]),
          };
          stmts.push(node::Instruction::AsyncCall(func_call));
        } else {
          // async <func-id> { <id>: <expr>, ... }
          let call = content.parse::<node::FuncCall>()?;
          stmts.push(node::Instruction::AsyncCall(call));
        }
      } else if content.peek(syn::Ident) {
        match content.cursor().ident().unwrap().0.to_string().as_str() {
          // when <cond> { ... }
          "when" => {
            content.parse::<syn::Ident>()?;
            // TODO(@were): To keep it simple, for now, only a ident is allowed.
            let cond = content.parse::<syn::Ident>()?;
            let body = content.parse::<node::Body>()?;
            stmts.push(node::Instruction::When((cond, Box::new(body))));
            continue; // NO ;
          }
          // spin <array-ptr> <func-id> { <id>: <expr> }
          "spin" => {
            content.parse::<syn::Ident>()?; // spin
            let lock = content.parse::<node::ArrayAccess>()?;
            let call = content.parse::<node::FuncCall>()?;
            stmts.push(node::Instruction::SpinCall((lock, call)));
          }
          _ => {
            // Parse non-keyword-leading statements
            if content.peek2(syn::token::Bracket) {
              // <id>[<expr>] = <expr>
              let aa = content.parse::<node::ArrayAccess>()?;
              content.parse::<syn::Token![=]>()?;
              let right = content.parse::<syn::Expr>()?;
              stmts.push(node::Instruction::ArrayAssign((aa, right)));
            } else {
              // <id> = <expr>
              let id = content.parse::<Ident>()?;
              if content.peek(syn::Token![=]) {
                content.parse::<syn::Token![=]>()?;
                // to handle the expression in k = a[0.int::<32>]
                if content.peek(syn::Ident) && content.peek2(syn::token::Bracket) {
                  let aa = content.parse::<node::ArrayAccess>()?;
                  stmts.push(node::Instruction::ArrayRead((id, aa)));
                } else if {
                  // <id> = array(<ty>, <size>);
                  if let Some((id, _)) = content.cursor().ident() {
                    id.to_string() == "array"
                  } else {
                    false
                  }
                } {
                  content.parse::<syn::Ident>()?; // array
                  let args;
                  syn::parenthesized!(args in content);
                  let ty = args.parse::<DType>()?;
                  args.parse::<syn::Token![,]>()?;
                  let size = args.parse::<syn::LitInt>()?;
                  stmts.push(node::Instruction::ArrayAlloc((id, ty, size)));
                } else {
                  let assign = content.parse::<syn::Expr>()?;
                  stmts.push(node::Instruction::Assign((id, assign)));
                }
              } else {
                return Err(syn::Error::new(
                  content.span(),
                  "Expected an assignment or an expression",
                ));
              }
            }
          }
        }
      }
      content.parse::<syn::Token![;]>()?;
    }
    Ok(node::Body { stmts })
  }
}
