use num_bigint::{BigInt, BigUint, Sign};

pub trait ValueCastTo<T> {
  fn cast(&self) -> T;
}

#[inline]
fn low_u64_from_biguint(value: &BigUint) -> u64 {
  value.iter_u64_digits().next().unwrap_or(0)
}

#[inline]
fn low_u64_from_bigint_bits(value: &BigInt) -> u64 {
  let low = low_u64_from_biguint(value.magnitude());
  if value.sign() == Sign::Minus {
    low.wrapping_neg()
  } else {
    low
  }
}

#[inline]
fn biguint_from_bigint_bits(value: &BigInt) -> BigUint {
  match value.sign() {
    Sign::NoSign => BigUint::from(0u8),
    Sign::Plus => value.magnitude().clone(),
    Sign::Minus => BigUint::from(low_u64_from_bigint_bits(value)),
  }
}

macro_rules! impl_bool_target {
  ($target:ty) => {
    impl ValueCastTo<$target> for bool {
      fn cast(&self) -> $target {
        if *self {
          1 as $target
        } else {
          0 as $target
        }
      }
    }
  };
}

impl ValueCastTo<bool> for bool {
  fn cast(&self) -> bool {
    *self
  }
}

impl ValueCastTo<BigInt> for bool {
  fn cast(&self) -> BigInt {
    BigInt::from(if *self { 1u8 } else { 0u8 })
  }
}

impl ValueCastTo<BigUint> for bool {
  fn cast(&self) -> BigUint {
    BigUint::from(if *self { 1u8 } else { 0u8 })
  }
}

impl_bool_target!(u8);
impl_bool_target!(u16);
impl_bool_target!(u32);
impl_bool_target!(u64);
impl_bool_target!(i8);
impl_bool_target!(i16);
impl_bool_target!(i32);
impl_bool_target!(i64);

macro_rules! impl_unsigned_source {
  ($source:ty) => {
    impl ValueCastTo<bool> for $source {
      fn cast(&self) -> bool {
        *self != 0
      }
    }

    impl ValueCastTo<BigInt> for $source {
      fn cast(&self) -> BigInt {
        BigInt::from(*self)
      }
    }

    impl ValueCastTo<BigUint> for $source {
      fn cast(&self) -> BigUint {
        BigUint::from(*self)
      }
    }

    impl_primitive_targets_for_source!($source);
  };
}

macro_rules! impl_signed_source {
  ($source:ty, $unsigned:ty) => {
    impl ValueCastTo<bool> for $source {
      fn cast(&self) -> bool {
        *self != 0
      }
    }

    impl ValueCastTo<BigInt> for $source {
      fn cast(&self) -> BigInt {
        BigInt::from(*self)
      }
    }

    impl ValueCastTo<BigUint> for $source {
      fn cast(&self) -> BigUint {
        BigUint::from(*self as $unsigned)
      }
    }

    impl_primitive_targets_for_source!($source);
  };
}

macro_rules! impl_primitive_targets_for_source {
  ($source:ty) => {
    impl ValueCastTo<u8> for $source {
      fn cast(&self) -> u8 {
        *self as u8
      }
    }

    impl ValueCastTo<u16> for $source {
      fn cast(&self) -> u16 {
        *self as u16
      }
    }

    impl ValueCastTo<u32> for $source {
      fn cast(&self) -> u32 {
        *self as u32
      }
    }

    impl ValueCastTo<u64> for $source {
      fn cast(&self) -> u64 {
        *self as u64
      }
    }

    impl ValueCastTo<i8> for $source {
      fn cast(&self) -> i8 {
        *self as i8
      }
    }

    impl ValueCastTo<i16> for $source {
      fn cast(&self) -> i16 {
        *self as i16
      }
    }

    impl ValueCastTo<i32> for $source {
      fn cast(&self) -> i32 {
        *self as i32
      }
    }

    impl ValueCastTo<i64> for $source {
      fn cast(&self) -> i64 {
        *self as i64
      }
    }
  };
}

impl_unsigned_source!(u8);
impl_unsigned_source!(u16);
impl_unsigned_source!(u32);
impl_unsigned_source!(u64);
impl_signed_source!(i8, u8);
impl_signed_source!(i16, u16);
impl_signed_source!(i32, u32);
impl_signed_source!(i64, u64);

impl ValueCastTo<bool> for BigUint {
  fn cast(&self) -> bool {
    self.iter_u64_digits().next().is_some()
  }
}

impl ValueCastTo<BigInt> for BigUint {
  fn cast(&self) -> BigInt {
    BigInt::from_biguint(Sign::Plus, self.clone())
  }
}

impl ValueCastTo<BigUint> for BigUint {
  fn cast(&self) -> BigUint {
    self.clone()
  }
}

macro_rules! impl_biguint_to_primitive {
  ($target:ty) => {
    impl ValueCastTo<$target> for BigUint {
      fn cast(&self) -> $target {
        low_u64_from_biguint(self) as $target
      }
    }
  };
}

impl_biguint_to_primitive!(u8);
impl_biguint_to_primitive!(u16);
impl_biguint_to_primitive!(u32);
impl_biguint_to_primitive!(u64);
impl_biguint_to_primitive!(i8);
impl_biguint_to_primitive!(i16);
impl_biguint_to_primitive!(i32);
impl_biguint_to_primitive!(i64);

impl ValueCastTo<bool> for BigInt {
  fn cast(&self) -> bool {
    self.sign() != Sign::NoSign
  }
}

impl ValueCastTo<BigInt> for BigInt {
  fn cast(&self) -> BigInt {
    self.clone()
  }
}

impl ValueCastTo<BigUint> for BigInt {
  fn cast(&self) -> BigUint {
    biguint_from_bigint_bits(self)
  }
}

macro_rules! impl_bigint_to_primitive {
  ($target:ty) => {
    impl ValueCastTo<$target> for BigInt {
      fn cast(&self) -> $target {
        low_u64_from_bigint_bits(self) as $target
      }
    }
  };
}

impl_bigint_to_primitive!(u8);
impl_bigint_to_primitive!(u16);
impl_bigint_to_primitive!(u32);
impl_bigint_to_primitive!(u64);
impl_bigint_to_primitive!(i8);
impl_bigint_to_primitive!(i16);
impl_bigint_to_primitive!(i32);
impl_bigint_to_primitive!(i64);
