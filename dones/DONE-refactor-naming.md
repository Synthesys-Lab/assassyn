# TODO Document

## Goal

Replace "_inst" suffix with "Instance" suffix in module naming within the builder module to eliminate Rust compiler warnings and improve naming consistency.

## Action Items

1. **Update design document for module naming convention**
   - Update `python/assassyn/builder/type_oriented_namer.md` to document the new "Instance" suffix naming convention
   - Discuss the design change to ensure it's clear and reasonable
   - Commit the design document with message "Update design document for module naming convention"

2. **Create test case for new naming convention**
   - As per new design doc, create a new test case in `python/ci-tests/test_async_call.py` to verify modules are named with "Instance" suffix instead of "_inst"
   - Test should verify that `Adder` module becomes `AdderInstance` and `Driver` module becomes `DriverInstance`

3. **Refactor module prefix generation**
   - As per new design doc, modify `python/assassyn/builder/type_oriented_namer.py` in the `_module_prefix()` method (line 98)
   - Change `return f'{base}_inst'` to `return f'{base}Instance'`
   - This single line change will update all module naming from "_inst" to "Instance" suffix

4. **Verify the change eliminates compiler warnings**
   - Run `python python/ci-tests/test_async_call.py` to confirm generated Rust code uses proper PascalCase naming
   - Verify no "_inst" suffixes appear in generated code
   - Confirm Rust compiler warnings about non-snake_case module names are eliminated
