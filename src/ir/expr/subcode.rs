use std::fmt::{Display, Formatter};

macro_rules! register_subcode {

  ($namespace:ident { $($opcode:ident ( $method:ident $op_lit:literal )),* $(,)? } ) => {

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum $namespace {
      $( $opcode ),*
    }

    impl Display for $namespace {
      fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
          $( $namespace::$opcode => write!(f, $op_lit) ),*
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
    Mod(mod "%"),
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
