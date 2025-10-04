# Cast

This module provides a cross-product to cast `bool`, `u{8,16,32,64}`, `i{8,16,32,64}`,
`BigInt`, and `BigUint`, so that the code generator can cast among them using a
unified interface:

```rust
// T is the target type
// value is the source value
pub trait ValueCastTo<T> {
  fn cast(&self) -> T;
}
```
