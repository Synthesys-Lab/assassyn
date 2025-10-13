# TODO: Documentation Issues and Inconsistencies

## Goal

This document reports unclear parts, inconsistencies, and potential improvements identified during the documentation of the Verilog code generation modules. These issues require human intervention to resolve.

## Action Items

### 1. Unclear Project-Specific Knowledge

**Issue**: Several functions reference project-specific knowledge that could not be fully understood from the codebase:

- **External Module Integration**: The `ExternalSV` class and its integration with the Verilog backend has complex behavior that could benefit from clearer documentation
- **Credit-based Pipeline Details**: While the high-level architecture is documented, some implementation details of the credit system could be clearer
- **CIRCT Framework Integration**: The specific CIRCT operations and their usage patterns could be better documented

**Recommendation**: Create additional design documents or improve existing ones to clarify these concepts.

### 2. Function Name vs Implementation Inconsistencies

**Issue**: Some function names don't clearly reflect their actual behavior:

- **`cleanup_post_generation`**: The name suggests cleanup, but the function actually generates complex signal routing logic
- **`generate_sram_blackbox_files`**: The function generates both blackbox modules and regular SRAM modules, not just blackboxes
- **`dump_rval`**: The name suggests dumping, but it actually generates signal references

**Recommendation**: Consider renaming these functions to better reflect their actual behavior, or update the function names to match their current implementation.

### 3. Complex State Management

**Issue**: The `CIRCTDumper` class has extensive state management with many instance variables that could be better organized:

- **State Variables**: Over 20 instance variables make the class complex to understand
- **State Dependencies**: Some state variables depend on others in complex ways
- **State Lifecycle**: The lifecycle of state variables across different phases is not clearly documented

**Recommendation**: Consider refactoring the state management into smaller, more focused classes or provide better documentation of the state lifecycle.

### 4. Error Handling and Edge Cases

**Issue**: Several functions have limited error handling or unclear behavior for edge cases:

- **Array Write Port Assignment**: The logic for assigning write ports to arrays could fail silently in some cases
- **External Module Detection**: The detection of external modules relies on attributes that might not always be set
- **FIFO Depth Calculation**: The FIFO depth calculation logic could be more robust

**Recommendation**: Add more robust error handling and validation for these edge cases.

### 5. Documentation Gaps

**Issue**: Some important concepts are not well documented:

- **Multi-Port Array Architecture**: While implemented, the design decisions behind multi-port arrays could be better explained
- **FIFO Handshaking Protocols**: The specific handshaking protocols used are not clearly documented
- **External Module Integration**: The process of integrating external SystemVerilog modules could be clearer

**Recommendation**: Create additional documentation for these concepts or improve existing documentation.

### 6. Code Organization

**Issue**: Some modules have functions that could be better organized:

- **`design.py`**: The `CIRCTDumper` class is very large and could be split into smaller classes
- **`top.py`**: The `generate_top_harness` function is very long and could be broken down
- **`elaborate.py`**: The file handles multiple responsibilities that could be separated

**Recommendation**: Consider refactoring these modules to improve maintainability and readability.

## Summary

The documentation process revealed several areas where the codebase could be improved for better maintainability and clarity. The most critical issues are around function naming consistency and complex state management. These improvements would make the codebase easier to understand and maintain for future developers.
