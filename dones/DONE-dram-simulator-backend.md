# TODO: DRAM Simulator Backend Implementation

## Goal

Implement a new DRAM simulator backend that supports per-DRAM-module memory interfaces with proper callback handling and response management, replacing the current single global `MemoryInterface` approach.

## Action Items

### 0. Document Development

**0.1** Update design documents to reflect the new DRAM simulator backend design:
- Update `python/assassyn/codegen/simulator/_expr/intrinsics.md` to document new DRAM intrinsic code generation
- Update `python/assassyn/codegen/simulator/modules.md` to document DRAM module generation with callback handling
- Update `python/assassyn/codegen/simulator/simulator.md` to document per-DRAM memory interface generation
- Update `python/assassyn/ir/expr/intrinsic.md` to document new DRAM intrinsic functions
- Update `tools/rust-sim-runtime/src/ramulator2.md` to document enhanced runtime interface

**Commit message**: "Update design documents for DRAM simulator backend implementation"

### 1. Coding Development

**1.1** Update intrinsic frontend API in `python/assassyn/ir/expr/intrinsic.py`:
- Add missing intrinsic opcodes: `HAS_MEM_RESP` (904), `MEM_RESP` (907), `GET_MEM_RESP` (912)
- Implement frontend builder functions: `has_mem_resp()`, `mem_resp()`, `get_mem_resp()`
- Update `INTRIN_INFO` dictionary with proper metadata for new intrinsics
- Ensure all DRAM intrinsics have proper `@ir_builder` decorators
- Update `dtype` property to handle new intrinsic return types

**Commit message**: "Add DRAM intrinsic frontend API functions"

**1.2** Enhance simulator generation in `python/assassyn/codegen/simulator/simulator.py`:
- Update `analyze_and_register_ports()` to detect and register DRAM modules
- Modify `dump_simulator()` to generate per-DRAM memory interfaces instead of single global interface
- Add `mi_<dram_name>: MemoryInterface` and `<dram_name>_response: Response` fields
- Update simulator methods to handle multiple memory interfaces
- Implement DRAM callback registration and context management

**Commit message**: "Implement per-DRAM memory interface generation in simulator"

**1.3** Update intrinsic code generation in `python/assassyn/codegen/simulator/_expr/intrinsics.py`:
- Implement `codegen_intrinsic()` for DRAM operations (`SEND_READ_REQUEST`, `SEND_WRITE_REQUEST`, etc.)
- Generate callback functions for each DRAM module with proper Ramulator2 signatures
- Handle callback context management and memory interface references
- Add error handling for memory operation failures

**Commit message**: "Implement DRAM intrinsic code generation with callback support"

**1.4** Enhance Ramulator2 runtime interface in `tools/rust-sim-runtime/src/ramulator2.rs`:
- Update `Response` struct with `read_succ`, `write_succ`, and `is_write` fields
- Enhance `MemoryInterface` struct with `write_buffer` and response state management
- Implement response handling methods and proper state reset functionality
- Update callback handling with proper registration and thread safety
- Add multi-interface support for multiple `MemoryInterface` instances

**Commit message**: "Enhance Ramulator2 runtime interface for per-DRAM support"

**1.5** Update module generation in `python/assassyn/codegen/simulator/modules.py`:
- Implement DRAM module code generation with callback functions
- Update module visitor pattern to handle DRAM-specific generation
- Implement callback context management and response processing
- Add DRAM-specific error handling and debugging support

**Commit message**: "Implement DRAM module generation with callback handling"

**1.6** Port existing test_dram.py to new frontend API and move to ci-tests:
- Port `python/todo/test_dram.py` to use new DRAM frontend API with proper intrinsic functions
- Move the ported test to `python/ci-tests/test_dram.py` 
- Ensure test produces equivalent output to reference implementations:
  - `tools/c-ramulator2-wrapper/test.cpp` (C++ wrapper test)
  - `python/unit-tests/test_ramulator2.py` (Python Ramulator2 test)
  - `tools/rust-sim-runtime/tests/test_ramulator2.rs` (Rust runtime test)
- Verify test logic matches: counter-based read/write pattern, address calculation, callback handling
- Update test to use new DRAM intrinsics: `has_mem_resp()`, `get_mem_resp()`, proper callback registration

**Commit message**: "Port test_dram.py to new frontend API and move to ci-tests"

**1.7** Create additional test cases for DRAM simulator backend:
- Add test cases in `python/ci-tests/` for basic DRAM read/write operations
- Test multiple DRAM modules in same system
- Test callback handling and response processing
- Test error conditions and edge cases

**Commit message**: "Add additional test cases for DRAM simulator backend"

**1.8** Update existing test infrastructure:
- Modify test drivers to work with new DRAM backend
- Update test configurations for multiple memory interfaces
- Ensure all existing tests pass with new implementation

**Commit message**: "Update test infrastructure for DRAM simulator backend"

## Summary Checklist

### Goal Achieved
✅ Successfully implemented a new DRAM simulator backend that supports per-DRAM-module memory interfaces with proper callback handling and response management, replacing the previous single global `MemoryInterface` approach.

### Action Items Completed
- ✅ **0.1** Update design documents to reflect the new DRAM simulator backend design
- ✅ **1.1** Update intrinsic frontend API in `python/assassyn/ir/expr/intrinsic.py`
- ✅ **1.2** Enhance simulator generation in `python/assassyn/codegen/simulator/simulator.py`
- ✅ **1.3** Update intrinsic code generation in `python/assassyn/codegen/simulator/_expr/intrinsics.py`
- ✅ **1.4** Enhance Ramulator2 runtime interface in `tools/rust-sim-runtime/src/ramulator2.rs`
- ✅ **1.5** Update module generation in `python/assassyn/codegen/simulator/modules.py`
- ✅ **1.6** Port existing test_dram.py to new frontend API and move to ci-tests
- ✅ **1.7** Create additional test cases for DRAM simulator backend
- ✅ **1.8** Update existing test infrastructure

### Changes Made in Codebase

#### Improvements Made
- **Per-DRAM Memory Interfaces**: Replaced single global memory interface with per-DRAM-module interfaces (`mi_<dram_name>`)
- **Enhanced Response Management**: Added per-DRAM response fields (`<dram_name>_response`) with proper state tracking
- **Callback System Redesign**: Implemented per-DRAM callback functions (`callback_of_<dram_name>`) for better isolation
- **Multi-DRAM Support**: Added support for multiple DRAM modules in the same system
- **Enhanced Runtime Interface**: Updated Ramulator2 interface with write buffer and response state management

#### New Interfaces Created
- **DRAMCallbackMetadata**: New metadata structure for per-DRAM callback management
- **Response Handling Methods**: Added `reset_state()`, `add_write_data()`, `get_write_data()` methods to MemoryInterface
- **Per-DRAM Intrinsic Code Generation**: Updated intrinsic code generation to use per-DRAM memory interfaces

#### Interface Refactor Example
**Before (Single Global Interface):**
```rust
pub struct Simulator {
    pub mem_interface: MemoryInterface,
    // ...
}
```

**After (Per-DRAM Interfaces):**
```rust
pub struct Simulator {
    pub mi_dram1: MemoryInterface,
    pub dram1_response: Response,
    pub mi_dram2: MemoryInterface,
    pub dram2_response: Response,
    // ...
}
```

### Technical Decisions Made

#### Short-term Implementation Choices
- **Callback Function Naming**: Used `callback_of_<dram_name>` naming convention for per-DRAM callbacks
- **Response Field Structure**: Added `read_succ`, `write_succ`, and `is_write` fields to Response struct for better state tracking
- **Write Buffer Management**: Implemented VecDeque-based write buffer for memory order retention

#### Fundamental Solutions Recommended
- **Memory Interface Isolation**: The per-DRAM approach provides better isolation and prevents interference between different memory modules
- **Callback Context Management**: Each DRAM module now has its own callback context, eliminating shared state issues
- **Response State Tracking**: Enhanced response tracking allows for better debugging and error handling

#### Test Infrastructure Updates
- **Test Porting**: Successfully ported existing test_dram.py to use new DRAM intrinsics
- **Additional Test Coverage**: Added comprehensive test cases for multiple DRAM modules, error conditions, and edge cases
- **Test Infrastructure Compatibility**: Existing test infrastructure works seamlessly with new DRAM backend
