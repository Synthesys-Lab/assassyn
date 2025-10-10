# TODO: Eliminate Callback Collector and Fix DRAM Test

## Goal

Eliminate the `CallbackIntrinsicCollector` system and integrate callback generation directly into DRAM module generation, while fixing the DRAM test compilation issues. Additionally, change `has_mem_resp` and `get_mem_resp` from `Intrinsic` to `PureIntrinsic` as they are purely combinational operations without side effects.

## Problem Analysis

The current DRAM test (`test_dram.py`) is failing to compile due to several issues:

### 1. Callback Scope Issues
- **Issue**: Generated code references `callback_of_DRAM_e0c81` which is not in scope
- **Root Cause**: Callbacks are generated in `mod.rs` but used in individual module files
- **Impact**: Compilation fails with "cannot find value" errors

### 2. Vec<u8> Conversion Problems
- **Issue**: `Vec<u8>` cannot be cast to `u64` using `ValueCastTo::<u64>::cast()`
- **Root Cause**: Missing proper conversion from `Vec<u8>` to `BigUint` as documented in `intrinsic.md`
- **Impact**: Compilation fails with trait bound errors

### 3. Display Formatting Issues
- **Issue**: `Vec<u8>` cannot be formatted with `{}` in Rust
- **Root Cause**: Missing proper conversion for display purposes
- **Impact**: Compilation fails with formatting errors

### 4. Architectural Issues
- **Issue**: `CallbackIntrinsicCollector` creates unnecessary complexity and separation of concerns
- **Root Cause**: Callback generation is separated from DRAM module generation
- **Impact**: Harder to maintain and debug, scope issues

### 5. Intrinsic Classification Issues
- **Issue**: `has_mem_resp` and `get_mem_resp` are classified as `Intrinsic` but should be `PureIntrinsic`
- **Root Cause**: They are purely combinational operations without side effects
- **Impact**: Incorrect semantic classification

## Action Items

### 1. Document Development

**1.1** Update design documents to reflect the new callback generation approach:
- Update `python/assassyn/codegen/simulator/modules.md` to document inline callback generation
- Update `python/assassyn/ir/expr/intrinsic.md` to clarify Vec<u8> conversion requirements
- Document the change from `Intrinsic` to `PureIntrinsic` for memory response operations

**Commit message**: "Update design documents for inline callback generation and PureIntrinsic classification"

### 2. Coding Development

**2.1** Change `has_mem_resp` and `get_mem_resp` to `PureIntrinsic`:
- Update `python/assassyn/ir/expr/intrinsic.py` to move these operations to `PureIntrinsic` class
- Update the opcode constants and `INTRIN_INFO` mapping
- Update the `dtype` property to handle the new classification
- Update frontend builder functions to use `PureIntrinsic`

**Commit message**: "Change has_mem_resp and get_mem_resp to PureIntrinsic classification"

**2.2** Update code generation for PureIntrinsic memory operations:
- Update `python/assassyn/codegen/simulator/_expr/intrinsics.py` to handle the new PureIntrinsic operations
- Move the code generation functions from `_INTRINSIC_DISPATCH` to `_PURE_INTRINSIC_DISPATCH`
- Ensure proper handling of the Vec<u8> response data

**Commit message**: "Update code generation for PureIntrinsic memory response operations"

**2.3** Implement Vec<u8> to BigUint conversion:
- Update the `get_mem_resp` code generation to use `BigUint::from_bytes_le` as documented
- Add proper conversion utilities in the generated code
- Fix the display formatting for Vec<u8> in log statements

**Commit message**: "Implement Vec<u8> to BigUint conversion for memory responses"

**2.4** Eliminate CallbackIntrinsicCollector and integrate callback generation:
- Remove `python/assassyn/codegen/simulator/callback_collector.py`
- Update `python/assassyn/codegen/simulator/modules.py` to generate callbacks inline with DRAM modules
- Remove all imports and references to `CallbackIntrinsicCollector`
- Generate callback functions directly in the same file as the DRAM module that uses them

**Commit message**: "Eliminate CallbackIntrinsicCollector and integrate inline callback generation"

**2.5** Update DRAM module generation to include inline callbacks:
- Modify the DRAM module generation to include the callback function in the same file
- Ensure proper scoping and visibility of callback functions
- Update the callback function signature and implementation to match the new approach

**Commit message**: "Update DRAM module generation with inline callback functions"

**2.6** Fix the DRAM test implementation:
- Update `python/ci-tests/test_dram.py` to work with the new PureIntrinsic classification
- Fix any import issues and ensure proper usage of the updated intrinsics
- Add proper error checking and validation

**Commit message**: "Fix DRAM test to work with PureIntrinsic memory operations"

### 3. Validation and Testing

**3.1** Run the fixed DRAM test:
- Execute `python/ci-tests/test_dram.py` to ensure it compiles and runs successfully
- Verify that all DRAM operations work correctly with the new approach
- Check that callback functions are properly generated and scoped

**Commit message**: "Validate DRAM test with eliminated callback collector"

**3.2** Run comprehensive test suite:
- Execute `make test-all` to ensure no regressions
- Run `python/ci-tests/test_driver.py` as sanity check
- Verify that existing functionality still works with the PureIntrinsic changes

**Commit message**: "Run comprehensive test suite to validate PureIntrinsic changes"

**3.3** Clean up unused code and imports:
- Remove any remaining references to the callback collector system
- Clean up unused imports in generated Rust code
- Remove the `callback_collector.py` file

**Commit message**: "Clean up unused callback collector code and imports"

## Technical Details

### Expected Behavior After Fix

The DRAM test should work as follows:

```python
# Create DRAM module
dram = DRAM(width, 512, init_file)

# Build with proper parameters - returns success values directly
read_succ, write_succ = dram.build(we, re, addr, wdata)

# Use PureIntrinsic operations for memory responses
with Condition(read_succ & has_mem_resp(dram)):  # PureIntrinsic
    resp = get_mem_resp(dram)  # PureIntrinsic, returns Vec<u8>
    # Process response with proper Vec<u8> to BigUint conversion
```

### Code Generation Changes

1. **Inline Callback Generation**: Each DRAM module generates its own callback function in the same file
2. **PureIntrinsic Classification**: `has_mem_resp` and `get_mem_resp` are now PureIntrinsic operations
3. **Vec<u8> Conversion**: Proper conversion using `BigUint::from_bytes_le` as documented
4. **Simplified Architecture**: No separate callback collection phase

### API Changes

After the fix, the API will be:
- `send_read_request(mem, re, addr)` → `Intrinsic`, returns `bool` (success)
- `send_write_request(mem, we, addr, data)` → `Intrinsic`, returns `bool` (success)
- `has_mem_resp(mem)` → `PureIntrinsic`, returns `bool`
- `get_mem_resp(mem)` → `PureIntrinsic`, returns `Vec<u8>`

## Success Criteria

1. ✅ `test_dram.py` compiles successfully without errors
2. ✅ `test_dram.py` runs and passes all assertions
3. ✅ Callback functions are generated inline with DRAM modules
4. ✅ `has_mem_resp` and `get_mem_resp` are properly classified as PureIntrinsic
5. ✅ Vec<u8> to BigUint conversion works correctly
6. ✅ All existing tests continue to pass without regressions
7. ✅ `CallbackIntrinsicCollector` is completely eliminated
8. ✅ Code generation produces clean, correct Rust code

## Risk Assessment

- **Low Risk**: The changes are primarily architectural improvements and fixing existing issues
- **Medium Risk**: Changing Intrinsic to PureIntrinsic classification could affect other parts of the system
- **Mitigation**: Comprehensive testing with existing test suite to catch any regressions

## Dependencies

- Existing DRAM simulator backend implementation
- Ramulator2 runtime interface
- Current test infrastructure
- Code generation infrastructure

## Estimated Effort

- **Document Updates**: 1 hour
- **PureIntrinsic Classification**: 2-3 hours
- **Callback Collector Elimination**: 3-4 hours
- **Vec<u8> Conversion Implementation**: 2-3 hours
- **Test Fixes and Validation**: 2-3 hours

**Total Estimated Time**: 10-14 hours

## Summary

This TODO addresses the architectural issues with the callback collector system and fixes the DRAM test compilation problems. The solution involves eliminating the separate callback collection phase, integrating callback generation directly into DRAM modules, properly classifying memory response operations as PureIntrinsic, and implementing correct Vec<u8> to BigUint conversion as documented in the intrinsic design document.
