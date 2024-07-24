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

register_subcode!(
  Unary {
    Flip(flip "!"),
    Neg(neg "-"),
  }
);

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

register_subcode!(
  FIFO {
    Peek(peek "peek"),
    Valid(valid "valid"),
    AlmostFull(almost_full "almost_full"),
  }
);

register_subcode!(
  Cast {
    BitCast(bitcast "bitcast"),
    SExt(sext "sext"),
    ZExt(zext "zext"),
  }
);

register_subcode!(
  BlockIntrinsic {
    Value(value "value"),
    Condition(condition "condition"),
    Cycled(cycled "cycled"),
    WaitUntil(wait_until "wait_until"),
  }
);
