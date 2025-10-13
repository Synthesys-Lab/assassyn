# TODO: Documentation Fix for Ramulator2 Module

## Goal

Fix documentation inconsistencies and unclear parts in the `ramulator2.py` module documentation to align with the new documentation standards and clarify project-specific knowledge.

## Action Items

### Document Development

1. **Clarify Module Integration with Assassyn Architecture**
   - The current documentation mentions "Module pipeline stage architecture" but doesn't clearly explain how PyRamulator integrates with Assassyn's [Module](../ir/module/module.md) concept
   - Need to clarify that PyRamulator is used by DRAM modules in the simulator backend, not as a Module itself
   - Reference the [simulator.md](../codegen/simulator/simulator.md) documentation for DRAM simulation integration

2. **Document Project-Specific Knowledge**
   - The `load_shared_library` function references macOS compatibility with `RTLD_GLOBAL` mode but the explanation could be clearer
   - Need to better explain why `RTLD_GLOBAL` is needed for "recursive shared object dependencies"
   - Clarify the relationship between the C++ wrapper and the core Ramulator2 library

3. **Fix Function Documentation Inconsistencies**
   - The `send_request` method documentation mentions "for read requests" in the callback parameter description, but callbacks are used for both read and write requests
   - The `Request` structure documentation is incomplete - it only shows a subset of the actual fields defined in the code
   - The `__del__` method documentation doesn't mention the conditional check for `self.obj`

### Coding Development

4. **Update Function Documentation**
   - Fix the `send_request` callback parameter description to clarify it's used for both read and write requests
   - Complete the `Request` structure documentation with all fields from the actual `ctypes.Structure` definition
   - Add proper explanation for the `__del__` method's conditional cleanup logic

5. **Enhance Cross-Platform Documentation**
   - The current documentation mentions cross-platform support but could be more specific about the actual platform detection logic
   - Add more details about the fallback mechanism for different library extensions
   - Clarify the relationship between the build process and library path detection

### Deal with Prior Changes

6. **Reference Design Documents**
   - The documentation should better reference the [simulator.md](../codegen/simulator/simulator.md) for DRAM simulation details
   - Link to the [module.md](../internal/module.md) for understanding how PyRamulator fits into the overall architecture
   - Reference the [pipeline.md](../internal/pipeline.md) for understanding the credit-based pipeline architecture

## Issues Found

### Unclear Project-Specific Knowledge

1. **Module vs. DRAM Integration**: The documentation mentions "Module pipeline stage architecture" but PyRamulator is actually used by DRAM modules, not as a Module itself. This needs clarification.

2. **macOS Compatibility**: The `RTLD_GLOBAL` mode explanation is vague. Need to understand why this is specifically needed for "recursive shared object dependencies" in the context of Ramulator2.

3. **Callback Usage**: The documentation suggests callbacks are only for read requests, but the implementation shows they're used for both read and write requests.

### Inconsistencies with Implementation

1. **Request Structure**: The documented fields don't match the complete `ctypes.Structure` definition in the code.

2. **Error Handling**: The `__del__` method has conditional logic that's not documented.

3. **Library Loading**: The cross-platform detection logic could be better explained.

## Human Intervention Required

- **Architecture Understanding**: Need human input on how PyRamulator specifically integrates with Assassyn's Module architecture
- **macOS Compatibility**: Need clarification on why `RTLD_GLOBAL` is specifically needed for Ramulator2
- **Callback Semantics**: Need confirmation on whether callbacks are used for both read and write requests or just read requests

## Potential Code Improvements

- Consider adding more detailed error messages for library loading failures
- The `Request` structure could benefit from better field documentation
- The callback mechanism could be more clearly documented with examples
