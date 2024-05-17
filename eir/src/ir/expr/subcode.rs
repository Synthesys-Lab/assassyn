use super::Opcode;

macro_rules! register_subcode {

  ($namespace:ident { $($opcode:ident ( $method:ident $op:literal )),* $(,)? } ) => {

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum $namespace {
      $( $opcode ),*
    }

    impl ToString for $namespace {
      fn to_string(&self) -> String {
        match self {
          $( $namespace::$opcode => $op.into() ),*
        }
      }
    }

    impl $namespace {
      pub fn from_str(s: &str) -> Option<$namespace> {
        match s {
          $( stringify!($method) => Some($namespace::$opcode), )*
          _ => None
        }
      }
    }

  };

}

register_subcode!(
  Binary {
    Add(add "+"),
    Sub(sub "-"),
    Mul(mul "*"),
    Shl(shl "<<"),
    Shr(shr ">>"),
    BitwiseOr(bitwise_or "|"),
    BitwiseAnd(bitwise_and "&"),
    BitwiseXor(bitwise_xor "^"),
  }
);

impl From<Binary> for Opcode {
  fn from(s: Binary) -> Self {
    Opcode::Binary { binop: s }
  }
}

register_subcode!(
  Unary {
    Flip(flip "!"),
    Neg(neg "-"),
  }
);

impl From<Unary> for Opcode {
  fn from(s: Unary) -> Self {
    Opcode::Unary { uop: s }
  }
}

register_subcode!(
  Compare {
   IGT(igt ">"),
   ILT(ilt "<"),
   IGE(ige ">="),
   ILE(ile "<="),
   EQ(eq "==" ),
   NEQ(neq "!="),
  }
);

impl From<Compare> for Opcode {
  fn from(s: Compare) -> Self {
    Opcode::Compare { cmp: s }
  }
}

register_subcode!(
  FIFO {
    Peek(peek "peek"),
    Valid(valid "valid"),
    AlmostFull(almost_full "almost_full"),
  }
);

impl From<FIFO> for Opcode {
  fn from(s: FIFO) -> Self {
    Opcode::FIFOField { field: s }
  }
}

register_subcode!(
  Cast {
    Cast(cast "cast"),
    SExt(sext "sext"),
    ZExt(zext "zext"),
  }
);

impl From<Cast> for Opcode {
  fn from(s: Cast) -> Self {
    Opcode::Cast { cast: s }
  }
}
