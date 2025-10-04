use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};

pub trait ValueCastTo<T> {
  fn cast(&self) -> T;
}

impl ValueCastTo<bool> for bool {
  fn cast(&self) -> bool {
    self.clone()
  }
}
impl ValueCastTo<BigInt> for BigInt {
  fn cast(&self) -> BigInt {
    self.clone()
  }
}
impl ValueCastTo<BigUint> for BigInt {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<BigInt> for bool {
  fn cast(&self) -> BigInt {
    if *self {
      1.to_bigint().unwrap()
    } else {
      0.to_bigint().unwrap()
    }
  }
}
impl ValueCastTo<bool> for BigInt {
  fn cast(&self) -> bool {
    !self.eq(&0.to_bigint().unwrap())
  }
}
impl ValueCastTo<BigUint> for BigUint {
  fn cast(&self) -> BigUint {
    self.clone()
  }
}
impl ValueCastTo<BigInt> for BigUint {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for bool {
  fn cast(&self) -> BigUint {
    if *self {
      1.to_biguint().unwrap()
    } else {
      0.to_biguint().unwrap()
    }
  }
}
impl ValueCastTo<bool> for BigUint {
  fn cast(&self) -> bool {
    !self.eq(&0.to_biguint().unwrap())
  }
}
impl ValueCastTo<bool> for u8 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<u8> for bool {
  fn cast(&self) -> u8 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for u8 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for u8 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<u8> for BigInt {
  fn cast(&self) -> u8 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as u8,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (u8::MAX as u64)) as u8,
      num_bigint::Sign::NoSign => data[0] as u8,
    }
  }
}
impl ValueCastTo<u8> for BigUint {
  fn cast(&self) -> u8 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as u8;
    }
  }
}
impl ValueCastTo<u8> for u8 {
  fn cast(&self) -> u8 {
    self.clone()
  }
}
impl ValueCastTo<u16> for u8 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for u8 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for u8 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for u8 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for u8 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for u8 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for u8 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for u16 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<u16> for bool {
  fn cast(&self) -> u16 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for u16 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for u16 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<u16> for BigInt {
  fn cast(&self) -> u16 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as u16,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (u16::MAX as u64)) as u16,
      num_bigint::Sign::NoSign => data[0] as u16,
    }
  }
}
impl ValueCastTo<u16> for BigUint {
  fn cast(&self) -> u16 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as u16;
    }
  }
}
impl ValueCastTo<u8> for u16 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for u16 {
  fn cast(&self) -> u16 {
    self.clone()
  }
}
impl ValueCastTo<u32> for u16 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for u16 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for u16 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for u16 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for u16 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for u16 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for u32 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<u32> for bool {
  fn cast(&self) -> u32 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for u32 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for u32 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<u32> for BigInt {
  fn cast(&self) -> u32 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as u32,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (u32::MAX as u64)) as u32,
      num_bigint::Sign::NoSign => data[0] as u32,
    }
  }
}
impl ValueCastTo<u32> for BigUint {
  fn cast(&self) -> u32 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as u32;
    }
  }
}
impl ValueCastTo<u8> for u32 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for u32 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for u32 {
  fn cast(&self) -> u32 {
    self.clone()
  }
}
impl ValueCastTo<u64> for u32 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for u32 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for u32 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for u32 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for u32 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for u64 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<u64> for bool {
  fn cast(&self) -> u64 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for u64 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for u64 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<u64> for BigInt {
  fn cast(&self) -> u64 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as u64,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (u64::MAX as u64)) as u64,
      num_bigint::Sign::NoSign => data[0] as u64,
    }
  }
}
impl ValueCastTo<u64> for BigUint {
  fn cast(&self) -> u64 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as u64;
    }
  }
}
impl ValueCastTo<u8> for u64 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for u64 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for u64 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for u64 {
  fn cast(&self) -> u64 {
    self.clone()
  }
}
impl ValueCastTo<i8> for u64 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for u64 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for u64 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for u64 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for i8 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<i8> for bool {
  fn cast(&self) -> i8 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for i8 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for i8 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<i8> for BigInt {
  fn cast(&self) -> i8 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as i8,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (i8::MAX as u64)) as i8,
      num_bigint::Sign::NoSign => data[0] as i8,
    }
  }
}
impl ValueCastTo<i8> for BigUint {
  fn cast(&self) -> i8 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as i8;
    }
  }
}
impl ValueCastTo<u8> for i8 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for i8 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for i8 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for i8 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for i8 {
  fn cast(&self) -> i8 {
    self.clone()
  }
}
impl ValueCastTo<i16> for i8 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for i8 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for i8 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for i16 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<i16> for bool {
  fn cast(&self) -> i16 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for i16 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for i16 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<i16> for BigInt {
  fn cast(&self) -> i16 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as i16,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (i16::MAX as u64)) as i16,
      num_bigint::Sign::NoSign => data[0] as i16,
    }
  }
}
impl ValueCastTo<i16> for BigUint {
  fn cast(&self) -> i16 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as i16;
    }
  }
}
impl ValueCastTo<u8> for i16 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for i16 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for i16 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for i16 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for i16 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for i16 {
  fn cast(&self) -> i16 {
    self.clone()
  }
}
impl ValueCastTo<i32> for i16 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for i16 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for i32 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<i32> for bool {
  fn cast(&self) -> i32 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for i32 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for i32 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<i32> for BigInt {
  fn cast(&self) -> i32 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as i32,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (i32::MAX as u64)) as i32,
      num_bigint::Sign::NoSign => data[0] as i32,
    }
  }
}
impl ValueCastTo<i32> for BigUint {
  fn cast(&self) -> i32 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as i32;
    }
  }
}
impl ValueCastTo<u8> for i32 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for i32 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for i32 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for i32 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for i32 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for i32 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for i32 {
  fn cast(&self) -> i32 {
    self.clone()
  }
}
impl ValueCastTo<i64> for i32 {
  fn cast(&self) -> i64 {
    *self as i64
  }
}
impl ValueCastTo<bool> for i64 {
  fn cast(&self) -> bool {
    *self != 0
  }
}
impl ValueCastTo<i64> for bool {
  fn cast(&self) -> i64 {
    if *self {
      1
    } else {
      0
    }
  }
}
impl ValueCastTo<BigInt> for i64 {
  fn cast(&self) -> BigInt {
    self.to_bigint().unwrap()
  }
}
impl ValueCastTo<BigUint> for i64 {
  fn cast(&self) -> BigUint {
    self.to_biguint().unwrap()
  }
}
impl ValueCastTo<i64> for BigInt {
  fn cast(&self) -> i64 {
    let (sign, data) = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    }
    match sign {
      num_bigint::Sign::Plus => data[0] as i64,
      num_bigint::Sign::Minus => ((!data[0] + 1) & (i64::MAX as u64)) as i64,
      num_bigint::Sign::NoSign => data[0] as i64,
    }
  }
}
impl ValueCastTo<i64> for BigUint {
  fn cast(&self) -> i64 {
    let data = self.to_u64_digits();
    if data.is_empty() {
      return 0;
    } else {
      return data[0] as i64;
    }
  }
}
impl ValueCastTo<u8> for i64 {
  fn cast(&self) -> u8 {
    *self as u8
  }
}
impl ValueCastTo<u16> for i64 {
  fn cast(&self) -> u16 {
    *self as u16
  }
}
impl ValueCastTo<u32> for i64 {
  fn cast(&self) -> u32 {
    *self as u32
  }
}
impl ValueCastTo<u64> for i64 {
  fn cast(&self) -> u64 {
    *self as u64
  }
}
impl ValueCastTo<i8> for i64 {
  fn cast(&self) -> i8 {
    *self as i8
  }
}
impl ValueCastTo<i16> for i64 {
  fn cast(&self) -> i16 {
    *self as i16
  }
}
impl ValueCastTo<i32> for i64 {
  fn cast(&self) -> i32 {
    *self as i32
  }
}
impl ValueCastTo<i64> for i64 {
  fn cast(&self) -> i64 {
    self.clone()
  }
}
