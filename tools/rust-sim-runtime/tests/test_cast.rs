use sim_runtime::num_bigint::{BigInt, BigUint};
use sim_runtime::ValueCastTo;

#[test]
fn bool_and_scalar_casts_follow_bit_vector_rules() {
  assert!(ValueCastTo::<bool>::cast(&1u8));
  assert!(!ValueCastTo::<bool>::cast(&0u32));
  assert_eq!(ValueCastTo::<u8>::cast(&true), 1);
  assert_eq!(ValueCastTo::<i32>::cast(&false), 0);

  assert_eq!(ValueCastTo::<u8>::cast(&0x1234u16), 0x34);
  assert_eq!(ValueCastTo::<i8>::cast(&0xffu8), -1);
  assert_eq!(ValueCastTo::<u16>::cast(&-1i16), 0xffff);
  assert_eq!(ValueCastTo::<u8>::cast(&-2i8), 0xfe);
}

#[test]
fn negative_signed_scalars_cast_to_biguint_without_panicking() {
  assert_eq!(ValueCastTo::<BigUint>::cast(&-1i8), BigUint::from(0xffu8));
  assert_eq!(ValueCastTo::<BigUint>::cast(&-2i16), BigUint::from(0xfffeu16));
  assert_eq!(ValueCastTo::<BigUint>::cast(&-1i32), BigUint::from(0xffff_ffffu32));
  assert_eq!(ValueCastTo::<BigUint>::cast(&-1i64), BigUint::from(u64::MAX));
}

#[test]
fn biguint_to_scalars_uses_low_bits() {
  let wide = (BigUint::from(1u64) << 80usize) | BigUint::from(0x1234_80ffu64);

  assert_eq!(ValueCastTo::<u8>::cast(&wide), 0xff);
  assert_eq!(ValueCastTo::<i8>::cast(&wide), -1);
  assert_eq!(ValueCastTo::<u16>::cast(&wide), 0x80ff);
  assert_eq!(ValueCastTo::<i16>::cast(&wide), -32513);
  assert_eq!(ValueCastTo::<u32>::cast(&wide), 0x1234_80ff);
}

#[test]
fn negative_bigint_to_scalars_uses_twos_complement_low_bits() {
  let minus_one = BigInt::from(-1);
  assert_eq!(ValueCastTo::<u8>::cast(&minus_one), 0xff);
  assert_eq!(ValueCastTo::<i8>::cast(&minus_one), -1);
  assert_eq!(ValueCastTo::<u16>::cast(&minus_one), 0xffff);
  assert_eq!(ValueCastTo::<i16>::cast(&minus_one), -1);
  assert_eq!(ValueCastTo::<u32>::cast(&minus_one), 0xffff_ffff);
  assert_eq!(ValueCastTo::<i32>::cast(&minus_one), -1);
  assert_eq!(ValueCastTo::<u64>::cast(&minus_one), u64::MAX);
  assert_eq!(ValueCastTo::<i64>::cast(&minus_one), -1);

  let wide_negative = -((BigInt::from(1u64) << 80usize) + BigInt::from(5u8));
  assert_eq!(ValueCastTo::<u8>::cast(&wide_negative), 0xfb);
  assert_eq!(ValueCastTo::<i8>::cast(&wide_negative), -5);
  assert_eq!(ValueCastTo::<u64>::cast(&wide_negative), u64::MAX - 4);
  assert_eq!(ValueCastTo::<i64>::cast(&wide_negative), -5);
}

#[test]
fn bigint_to_biguint_is_panic_free_for_negative_values() {
  assert_eq!(ValueCastTo::<BigUint>::cast(&BigInt::from(-1)), BigUint::from(u64::MAX));
  assert_eq!(ValueCastTo::<BigUint>::cast(&BigInt::from(-2)), BigUint::from(u64::MAX - 1));
}
