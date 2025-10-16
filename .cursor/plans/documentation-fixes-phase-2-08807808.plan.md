<!-- 08807808-717d-4039-abf4-fb1197111591 cbdaed24-007d-4df0-a810-8ec10463aec0 -->
# Phase 2: Documentation Fixes and Code Improvements

## Overview

Phase 1 successfully documented the majority of Python modules in Assassyn (53/74 files completed). Phase 2 focuses on resolving the issues, inconsistencies, and unclear aspects identified during the documentation process. This phase addresses both documentation improvements and code quality enhancements.

## Core Issues to Address

### 1. Naming and Terminology Consistency

**Issue**: The term "Module" is used inconsistently throughout the codebase - sometimes referring to pipeline stages, sometimes to general modules.

**Affected Files**:

- `python/assassyn/ir/module/module.py` and `module.md`
- `python/assassyn/experimental/frontend/module.py` and `module.md`
- Design documents referencing "Module" vs "pipeline stage"

**Actions**:

- Clarify distinction between "Module" (class name) and "pipeline stage" (architectural concept)
- Add terminology section to documentation explaining legacy naming issue
- Update cross-references to use consistent terminology
- Consider long-term refactoring to rename Module to PipelineStage (breaking change)

### 2. DRAM Integration and Callback Implementation

**Issue**: DRAM callback implementation has unclear data handling and potential correctness issues.

**Affected Files**:

- `python/assassyn/codegen/simulator/modules.py`
- `python/assassyn/ramulator2/ramulator2.py`
- `python/assassyn/ir/memory/dram.py`

**Actions**:

- Verify DRAM callback data extraction logic (lines 201-202 in modules.py)
- Clarify memory response data format (Vec<u8> to BigUint conversion)
- Document relationship between request address and response data
- Update parameter order inconsistency in DRAM documentation
- Verify callback usage for both read and write requests

### 3. Type System and Error Handling Documentation

**Issue**: Type annotations and error handling inconsistencies across expression modules.

**Affected Files**:

- `python/assassyn/ir/expr/array.py` - Slice class type annotations
- `python/assassyn/ir/expr/arith.py` - carry bit handling
- `python/assassyn/ir/dtype.py` - Record.attributize limitations
- `python/assassyn/ir/const.py` - 32-bit limitation

**Actions**:

- Fix Slice class type documentation (claims int but returns UInt)
- Document carry bit handling decision in BinaryOp
- Document Record.attributize incomplete implementation
- Clarify 32-bit limitation in Const class
- Add "Error Conditions" sections to all expression modules

### 4. Runtime Validation and Safety Checks

**Issue**: Several methods lack runtime validation despite documented restrictions.

**Affected Files**:

- `python/assassyn/ir/module/base.py` - triggered() method
- `python/assassyn/ir/block.py` - block kind constants
- `python/assassyn/builder/rewrite_assign.py` - AST rewriting failures

**Actions**:

- Add runtime check to triggered() to enforce downstream-only usage
- Consider using enum for block kind constants instead of magic numbers
- Improve error reporting in AST rewriting failures
- Add validation for cycle numbers in CycledBlock
- Create test cases for new validations

### 5. Memory System Documentation Gaps

**Issue**: Memory initialization format and timing semantics are not well documented.

**Affected Files**:

- `python/assassyn/ir/memory/base.py`
- `python/assassyn/ir/memory/sram.py`
- `python/assassyn/ir/memory/dram.py`

**Actions**:

- Document memory initialization file format (.hex format, byte ordering)
- Clarify address width derivation logic (power-of-2 requirement)
- Document SRAM read data timing relationship
- Add cross-references to ramulator2 integration
- Document memory module naming conventions

### 6. Port and Pin Terminology

**Issue**: "Port" and "pin" are used inconsistently across experimental frontend.

**Affected Files**:

- `python/assassyn/experimental/frontend/module.py`
- `python/assassyn/experimental/frontend/downstream.py`
- `python/assassyn/experimental/frontend/factory.py`

**Actions**:

- Add clear definitions distinguishing ports from pins
- Update all documentation to use consistent terminology
- Explain when to use ports vs pins in different contexts
- Document systolic vs backpressure timing modes
- Resolve "Callback" module type RFC question

### 7. Verilog Code Generation Improvements

**Issue**: Function names don't clearly reflect behavior and state management is complex.

**Affected Files**:

- `python/assassyn/codegen/verilog/cleanup.py`
- `python/assassyn/codegen/verilog/design.py`
- `python/assassyn/codegen/verilog/elaborate.py`

**Actions**:

- Consider renaming cleanup_post_generation to reflect actual behavior
- Document CIRCTDumper state lifecycle
- Improve error handling for array write port assignment
- Document multi-port array architecture decisions
- Consider refactoring large functions (generate_top_harness, dump_simulator)

### 8. Utility Functions and Environment Dependencies

**Issue**: Error handling and environment variable requirements not well documented.

**Affected Files**:

- `python/assassyn/utils.py`
- `python/assassyn/backend.py`

**Actions**:

- Document FIFO patching rationale
- Document simulation output format for cycle parsing
- Clarify ASSASSYN_HOME and VERILATOR_ROOT requirements
- Document thread safety considerations
- Improve error messages for environment setup issues

### 9. Simulator Code Generation Refactoring

**Issue**: dump_simulator function violates single responsibility principle.

**Affected Files**:

- `python/assassyn/codegen/simulator/simulator.py`
- `python/assassyn/codegen/simulator/port_mapper.py`

**Actions**:

- Refactor dump_simulator into focused functions:
- generate_simulator_struct()
- generate_simulator_impl()
- generate_module_simulators()
- generate_main_simulation_loop()
- Document configuration parameter effects
- Add port mapper thread safety considerations
- Remove unused configuration parameters or implement functionality

### 10. Cross-Module Documentation Consistency

**Issue**: Cross-references and relationship documentation needs improvement.

**Affected Files**:

- All expression modules in `ir/expr/`
- All module files in `ir/module/`
- All codegen files

**Actions**:

- Add "Related Modules" sections to expression documentation
- Document inheritance hierarchies clearly
- Update cross-references to use correct .md extensions
- Add integration examples showing module interactions
- Ensure design document references are accurate

## Documentation Deliverables

### High Priority

1. Fix all type annotation inconsistencies
2. Resolve DRAM callback correctness issues
3. Clarify Module vs pipeline stage terminology
4. Document memory initialization format
5. Add runtime validation where needed

### Medium Priority

6. Improve error handling documentation
7. Document port vs pin distinction
8. Refactor large simulator generation functions
9. Improve utility function error messages
10. Document environment setup requirements

### Low Priority

11. Consider enum refactoring for constants
12. Improve thread safety documentation
13. Document performance characteristics
14. Add comprehensive usage examples
15. Review code organization opportunities

## Testing Requirements

- Add unit tests for new runtime validations
- Test DRAM callback with various configurations
- Validate memory module integration scenarios
- Test error handling for environment setup
- Ensure simulator generation produces valid Rust code

## Success Criteria

- All type inconsistencies resolved
- DRAM integration verified and documented
- Runtime validations in place with tests
- Terminology clearly defined throughout
- Error handling comprehensively documented
- All cross-references accurate and complete
- No remaining "unclear aspects" in TODO files

### To-dos

- [ ] Clarify Module vs pipeline stage terminology and update documentation
- [ ] Verify and fix DRAM callback data handling and parameter order
- [ ] Fix type annotation inconsistencies in expression modules
- [ ] Add runtime validation checks with test coverage
- [ ] Document memory initialization format and timing semantics
- [ ] Clarify port vs pin distinction in experimental frontend
- [ ] Improve error handling documentation across all modules
- [ ] Refactor dump_simulator into focused functions
- [ ] Update all cross-references and add Related Modules sections