# Test Cast

`test_cast.rs` locks the simulator runtime casting semantics exposed through
`ValueCastTo::<T>::cast(&value)`.

The tests treat integer casts as fixed-width bit-vector operations:

- narrowing keeps the low target-width bits;
- signed targets reinterpret the low bits as two's-complement values;
- unsigned targets reinterpret negative signed sources at the source width;
- boolean casts use zero/non-zero;
- BigInt and BigUint scalar casts use the low 64-bit limb instead of requiring
  full-width allocation.

The negative signed-to-BigUint cases are regression tests for generated
simulator paths that convert signed values into unsigned bit-vector contexts
such as slices and concatenations.
