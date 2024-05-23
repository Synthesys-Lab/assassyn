use eir::ir::{DataType, Opcode};
use syn::{bracketed, parenthesized, parse::Parse, punctuated::Punctuated, Token};

use super::node::{ArrayAccess, FuncCall, WeakSpanned};

pub(crate) enum ExprTerm {
  Ident(syn::Ident),
  Const((DType, syn::LitInt)),
  StrLit(syn::LitStr),
  ArrayAccess(ArrayAccess),
  DType(DType),
}

pub(crate) struct ModuleAttrs {
  pub(crate) attrs: Punctuated<syn::Ident, Token![,]>,
}

impl Parse for ModuleAttrs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut attrs = Punctuated::new();
    loop {
      if !input.peek(Token![#]) {
        break;
      }
      input.parse::<Token![#]>()?;
      let raw_attr = input.parse::<syn::Ident>()?;
      attrs.push_value(
        match eir::ir::module::Attribute::from_string(&raw_attr.to_string()) {
          Some(_) => raw_attr,
          _ => {
            return Err(syn::Error::new(
              raw_attr.span(),
              format!(
                "{}:{}: Unsupported attribute: \"{}\"",
                file!(),
                line!(),
                raw_attr
              ),
            ))
          }
        },
      );
      if !input.peek(Token![,]) {
        break;
      }
      attrs.push_punct(input.parse::<Token![,]>()?);
    }
    Ok(ModuleAttrs { attrs })
  }
}

impl WeakSpanned for ExprTerm {
  fn span(&self) -> proc_macro2::Span {
    match self {
      ExprTerm::Ident(id) => id.span(),
      ExprTerm::Const((_, lit)) => lit.span(),
      ExprTerm::StrLit(lit) => lit.span(),
      ExprTerm::ArrayAccess(ArrayAccess { id, .. }) => id.span(),
      ExprTerm::DType(dtype) => dtype.span,
    }
  }
}

impl Parse for ExprTerm {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if DType::dtype_starts(&input) {
      let dtype = input.parse::<DType>()?;
      Ok(ExprTerm::DType(dtype))
    } else if input.peek(syn::LitStr) {
      let lit = input.parse::<syn::LitStr>()?;
      Ok(ExprTerm::StrLit(lit))
    } else if input.cursor().ident().is_some() {
      let id = input.parse::<syn::Ident>()?;
      if input.peek(syn::token::Bracket) {
        let raw_idx;
        syn::bracketed!(raw_idx in input);
        let idx = raw_idx.parse::<Expr>()?;
        let idx = Box::new(idx);
        Ok(ExprTerm::ArrayAccess(ArrayAccess { id, idx }))
      } else {
        Ok(ExprTerm::Ident(id))
      }
    } else if input.cursor().literal().is_some() {
      let lit = input.parse::<syn::LitInt>()?;
      let ty = if input.peek(syn::Token![.]) {
        input.parse::<syn::Token![.]>()?;
        input.parse::<DType>()?
      } else {
        DType {
          span: lit.span(),
          dtype: DataType::int_ty(32),
        }
      };
      Ok(ExprTerm::Const((ty, lit)))
    } else {
      Err(syn::Error::new(
        input.span(),
        "Expected identifier or literal",
      ))
    }
  }
}

/// The left value of an assignment.
pub(crate) enum LValue {
  IdentList(Punctuated<syn::Ident, Token![,]>),
  Ident(syn::Ident),
  ArrayAccess(ArrayAccess),
}

impl Parse for LValue {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(syn::Ident) {
      if input.peek2(syn::token::Bracket) {
        let aa = input
          .parse::<ArrayAccess>()
          .unwrap_or_else(|_| panic!("{}:{}: Failed to parse array access", file!(), line!()));
        Ok(LValue::ArrayAccess(aa))
      } else {
        let mut idents = Punctuated::new();
        loop {
          idents.push_value(
            input
              .parse::<syn::Ident>()
              .unwrap_or_else(|_| panic!("{}:{}: Failed to parse identifier", file!(), line!())),
          );
          if !input.peek(syn::Token![,]) {
            break;
          }
          idents.push_punct(input.parse::<syn::Token![,]>()?);
          if !input.peek(syn::Ident) {
            break;
          }
        }
        if idents.len() == 1 && !idents.trailing_punct() {
          Ok(LValue::Ident(idents.first().unwrap().clone()))
        } else {
          Ok(LValue::IdentList(idents))
        }
      }
    } else {
      Err(syn::Error::new(input.span(), "Expected an identifier"))
    }
  }
}

pub(crate) enum Expr {
  // ExprTerm . syn::Ident ( args )
  MethodCall((Box<Expr>, syn::Ident, Punctuated<Expr, Token![,]>)),
  // BinaryOp ( Expr, Expr, ... )
  BinaryReduce((syn::Ident, Punctuated<Expr, Token![,]>)),
  // "default" ExprTerm . "case" ( ExprTerm, ExprTerm )
  //                    . "case" ( ExprTerm, ExprTerm ) *
  Select((Box<Expr>, Vec<(Expr, Expr)>)),
  // "bind" FuncCall
  Bind(FuncCall),
  // "array" ( DType, syn::LitInt, Option<ExprTerm> )
  ArrayAlloc(
    (
      syn::Ident,
      DType,
      syn::LitInt,
      Option<Punctuated<ExprTerm, Token![,]>>,
      Vec<syn::Ident>,
    ),
  ),
  // ExprTerm
  Term(ExprTerm),
}

fn expr_terminates(input: &syn::parse::ParseStream) -> bool {
  input.peek(syn::token::Brace)
    || input.is_empty()
    || input.peek(syn::Token![;])
    || input.peek(syn::Token![,])
    || {
      input.cursor().punct().map_or(false, |(punct, next)| {
        punct.as_char() == '.' && next.ident().map_or(false, |(ident, _)| ident.eq("case"))
      })
    }
}

impl Parse for Expr {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let tok = input.parse::<ExprTerm>()?;
    let preliminary = if let ExprTerm::Ident(id) = &tok {
      match id.to_string().as_str() {
        "default" => {
          let default_value = input.parse::<Expr>()?;
          let mut cases = Vec::new();
          while !input.peek(syn::Token![;]) {
            input.parse::<syn::Token![.]>()?; // Consume "."
            input.parse::<syn::Ident>()?; // Consume "case"
            let content;
            parenthesized!(content in input);
            let cond = content.parse::<Expr>()?;
            content.parse::<syn::Token![,]>().expect("Expect a \",\""); // Consume ","
            let value = content.parse::<Expr>()?;
            cases.push((cond, value));
          }
          return Ok(Expr::Select((Box::new(default_value), cases)));
        }
        "bind" => {
          let func = input.parse::<FuncCall>()?;
          return Ok(Expr::Bind(func));
        }
        "array" => {
          let args;
          syn::parenthesized!(args in input);
          let ty = args.parse::<DType>()?;
          args.parse::<syn::Token![,]>()?;
          let size = args.parse::<syn::LitInt>()?;
          let mut initializer = None;
          let mut attrs = Vec::new();
          while !args.is_empty() {
            args.parse::<syn::Token![,]>()?;
            if args.peek(syn::token::Bracket) {
              let raw_init;
              bracketed!(raw_init in args);
              let parsed = raw_init.parse_terminated(ExprTerm::parse, syn::Token![,])?;
              initializer = Some(parsed);
            } else if args.peek(syn::Token![#]) {
              args.parse::<syn::Token![#]>()?;
              attrs.push(args.parse::<syn::Ident>()?);
            }
          }
          return Ok(Expr::ArrayAlloc((id.clone(), ty, size, initializer, attrs)));
        }
        _ => {
          if if let Some(op) = Opcode::from_str(&id.to_string()) {
            op.arity().map_or(false, |x| x == 2)
          } else {
            false
          } {
            if !input.peek(syn::token::Paren) {
              return Err(syn::Error::new(
                id.span(),
                format!(
                  "{}:{}: Expected a pair of parentheses for binary reduce.",
                  file!(),
                  line!(),
                ),
              ));
            }
            let raw_operands;
            parenthesized!(raw_operands in input);
            let operands = raw_operands.parse_terminated(Expr::parse, syn::Token![,])?;
            if operands.len() < 2 {
              return Err(syn::Error::new(
                id.span(),
                format!("{}:{}: At least 2 operands to reduce!", file!(), line!(),),
              ));
            }
            Expr::BinaryReduce((id.clone(), operands))
          } else {
            Expr::Term(tok)
          }
        }
      }
    } else {
      Expr::Term(tok)
    };
    if expr_terminates(&input) {
      return Ok(preliminary);
    }
    let mut expr = preliminary;
    while !expr_terminates(&input) {
      match input.parse::<syn::Token![.]>() {
        // Consume "."
        Ok(_) => {}
        Err(_) => {
          Err(syn::Error::new(
            input.span(),
            format!("{}:{}: Expected \".\" or terminator.", file!(), line!(),),
          ))?;
        }
      }
      let operator = input.parse::<syn::Ident>()?;
      let raw_operands;
      parenthesized!(raw_operands in input);

      let op = Opcode::from_str(&operator.to_string());

      match op {
        Some(x) => {
          let operands = raw_operands.parse_terminated(Expr::parse, syn::Token![,])?;
          // Count the number of valued operands
          let valued_terms = operands
            .iter()
            .filter(|x| !matches!(x, Expr::Term(ExprTerm::DType(_))))
            .count();
          if x.arity().map_or(
            false,
            |x| valued_terms != x - 1, // "self" is not counted, so minus 1
          ) {
            return Err(syn::Error::new(
              operator.span(),
              format!(
                "{}:{}: Wrong operand number {} != {}.",
                file!(),
                line!(),
                x.arity().unwrap() - 1,
                operands.len()
              ),
            ));
          }
          expr = Expr::MethodCall((Box::new(expr), operator, operands));
        }
        None => {
          return Err(syn::Error::new(
            operator.span(),
            format!(
              "{}:{}: Unsupported operator: \"{}\"",
              file!(),
              line!(),
              operator
            ),
          ))
        }
      }
    }
    Ok(expr)
  }
}

#[derive(Clone)]
pub(crate) struct DType {
  pub(crate) span: proc_macro2::Span,
  pub(crate) dtype: DataType,
}

impl DType {
  pub(crate) fn dtype_starts(input: &syn::parse::ParseStream) -> bool {
    if let Some((id, _)) = input.cursor().ident() {
      matches!(id.to_string().as_str(), "int" | "uint" | "bits" | "module")
    } else {
      false
    }
  }
}

impl Parse for DType {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let span = input.cursor().span();
    let tyid = input.parse::<syn::Ident>()?;
    match tyid.to_string().as_str() {
      "int" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::int_ty(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "uint" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::uint_ty(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "bits" => {
        input.parse::<syn::Token![<]>()?;
        let bits = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![>]>()?;
        Ok(DType {
          span,
          dtype: DataType::raw_ty(bits.base10_parse::<usize>().unwrap()),
        })
      }
      "module" => {
        let args;
        parenthesized!(args in input);
        let parsed_args = args.parse_terminated(DType::parse, syn::Token![,])?;
        Ok(DType {
          span,
          dtype: DataType::module(parsed_args.iter().map(|x| x.dtype.clone()).collect()),
        })
      }
      _ => Err(syn::Error::new(
        tyid.span(),
        format!("[CG.Type] Unsupported type: {}", tyid),
      )),
    }
  }
}
