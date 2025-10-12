# TODO: Port Mapper Documentation Review

## Goal

Review and update the port_mapper.md documentation to comply with the new documentation standards, ensuring it provides comprehensive coverage of the port mapping functionality used in simulator code generation.

## Action Items

### Document Development

- [x] **Update documentation structure**: Restructured port_mapper.md to follow the new documentation standards with Section 1 (Exposed Interfaces) and Section 2 (Internal Helpers)
- [x] **Enhance function documentation**: Added detailed explanations for each exposed function, including their role in the code generation pipeline
- [x] **Add usage examples**: Included comprehensive examples showing multi-port array write scenarios and generated Rust code
- [x] **Document integration points**: Explained how the port mapper integrates with the simulator runtime and code generation phases

### Coding Development

- [x] **Verify implementation consistency**: Confirmed that the documentation accurately reflects the current implementation in port_mapper.py
- [x] **Check usage patterns**: Analyzed how the port mapper is used across the codebase in simulator.py, elaborate.py, and array.py

## Issues Identified

### 1. DRAM Callback Port Assignment

**Issue**: The documentation mentions DRAM callback writes using a reserved port name "DRAM_CALLBACK", but this functionality is not currently implemented in the codebase.

**Current State**: The code only handles regular module-to-array writes through the `ArrayWrite` IR node.

**Impact**: This is a documentation inconsistency that could mislead developers about DRAM integration capabilities.

**Recommendation**: Either implement DRAM callback port assignment or remove this section from the documentation until the feature is implemented.

### 2. Missing Test Coverage

**Issue**: There are no dedicated unit tests for the port mapper functionality in the ci-tests directory.

**Current State**: The port mapper is only tested indirectly through integration tests like `test_array_multi_write.py`.

**Impact**: Changes to port mapper logic could break without proper test coverage.

**Recommendation**: Add dedicated unit tests for `PortIndexManager` class methods and the singleton pattern behavior.

### 3. Global State Management

**Issue**: The port mapper uses a global singleton pattern which could lead to issues in multi-threaded environments or when multiple compilations run concurrently.

**Current State**: The `_port_manager` global variable is reset at the beginning of each compilation, but there's no thread safety.

**Impact**: Potential race conditions if multiple compilations run simultaneously.

**Recommendation**: Consider thread-local storage or explicit context passing for the port manager to avoid global state issues.

## Completed Actions

- [x] **Documentation restructured**: Updated port_mapper.md to follow new documentation standards
- [x] **Function explanations added**: Provided detailed explanations for all exposed interfaces
- [x] **Usage pipeline documented**: Explained the three-phase usage pattern (reset, analysis, code generation)
- [x] **Integration points clarified**: Documented how the port mapper works with the simulator runtime
- [x] **Examples provided**: Added comprehensive examples showing multi-port array write scenarios

## Next Steps

1. **Implement DRAM callback support**: If DRAM callback port assignment is needed, implement the "DRAM_CALLBACK" functionality
2. **Add unit tests**: Create dedicated test cases for port mapper functionality
3. **Consider thread safety**: Evaluate if global singleton pattern needs thread safety improvements
4. **Update related documentation**: Ensure other modules that use port mapper are properly documented

## Status

The port_mapper.md documentation has been successfully updated to comply with the new documentation standards. The main issues identified are related to missing functionality (DRAM callbacks) and test coverage, which should be addressed in future development cycles.
