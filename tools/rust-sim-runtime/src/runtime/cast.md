# Cast

This module provides the generated simulator's scalar cast surface. It supports
casts among `bool`, `u{8,16,32,64}`, `i{8,16,32,64}`, `BigInt`, and `BigUint`
through one trait:

```rust
// T is the target type
// value is the source value
pub trait ValueCastTo<T> {
  fn cast(&self) -> T;
}
```

## Semantics

Integer casts follow fixed-width bit-vector rules:

- narrowing keeps the low target-width bits;
- signed targets reinterpret the low bits as two's-complement values;
- unsigned targets reinterpret signed primitive sources at the source width;
- boolean casts use zero/non-zero;
- `BigInt` and `BigUint` scalar casts use the low 64-bit limb.

Negative signed primitive values converted to `BigUint` are panic-free. The cast
first reinterprets the source at its native width, so `-1i8` becomes `0xff`,
`-1i32` becomes `0xffff_ffff`, and `-1i64` becomes `u64::MAX`.

`BigInt` to `BigUint` preserves positive magnitudes. For negative `BigInt`
values, the width-free compatibility cast returns the low 64-bit
two's-complement value. Generated code that needs a wider signed-to-unsigned
reinterpretation should use an explicit width helper instead of relying on this
width-free cast.

## Helpers

`bigint_from_decimal` and `biguint_from_decimal` parse generated decimal
literals for values wider than Rust's primitive integer range. They are used by
the simulator code generator to avoid lossy `as i64` or `as u64` casts for
wide immediates.

`biguint_mask` builds an unsigned mask with the low `bits` bits set. Generated
wide slice code uses it instead of parsing a long `"111..."` string in the
simulator hot path.

`low_u64_from_biguint` reads the low limb with `iter_u64_digits()` so scalar
casts do not allocate a full digit vector.

`low_u64_from_bigint_bits` combines the sign and magnitude low limb. Negative
values use wrapping negation, which is the low 64-bit two's-complement
representation.

`biguint_from_bigint_bits` implements the width-free `BigInt` to `BigUint`
compatibility behavior described above.

Local macros generate the primitive cross-product impls. This keeps the public
`ValueCastTo` interface stable while avoiding repetitive hand-written impls.
