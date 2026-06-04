# Simulator BigInt Codegen Tests

`test_simulator_bigint_codegen.py` checks the generated Rust text for simulator
integer expressions that previously took heavy or lossy BigInt paths.

The tests cover:

- wide unsigned immediates using a `BigUint` decimal helper instead of
  truncating through `as u64`;
- wide signed immediates using a `BigInt` decimal helper instead of truncating
  through `as i64`;
- wide slice masks using a runtime numeric mask helper instead of
  `BigUint::parse_bytes("111...", 2)` in the generated hot path;
- scalar-width concatenation using a `u64` fast path;
- signed right shifts on non-power-of-two widths using the rounded Rust scalar
  type selected by `dtype_to_rust_type`.
