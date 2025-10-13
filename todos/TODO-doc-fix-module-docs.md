# TODO: Documentation Review for Module Files

## Goal

Complete documentation review and reorganization for the first four unchecked module files according to the new documentation standards, addressing inconsistencies and unclear aspects found during the review process.

## Action Items

### Documentation Development

1. **Address Legacy Naming Issue**: The documentation now correctly identifies that "Module" is actually a pipeline stage in the current architecture, but this is a legacy naming issue that should be addressed in the codebase. The term "Module" should be renamed to "PipelineStage" or similar to better reflect its actual purpose.

2. **Clarify FSM Debug Output**: The FSM class includes print statements for debug output during construction and generation. This should be documented as a development feature that may need to be made configurable or removed in production builds.

3. **External Module Integration**: The ExternalSV class has a TODO comment about using Verilator to convert external SV into dynamic links for the Rust simulator. This represents a potential future enhancement that should be tracked separately.

### Coding Development

1. **Module Naming Consistency**: 
   - **Current State**: The code uses "Module" to refer to what are actually pipeline stages
   - **Desired State**: Rename Module class to PipelineStage or Stage for clarity
   - **Impact**: This is a breaking change that would require updating all references throughout the codebase
   - **Test Cases**: All existing test cases would need to be updated to use the new naming

2. **FSM Debug Output Management**:
   - **Current State**: FSM class prints debug information unconditionally
   - **Desired State**: Make debug output configurable or remove in production
   - **Implementation**: Add a debug flag or use proper logging instead of print statements
   - **Test Cases**: Ensure FSM tests still pass with debug output disabled

3. **External Module Path Resolution**:
   - **Current State**: ExternalSV preserves relative paths until elaboration
   - **Desired State**: Document the path resolution strategy more clearly
   - **Implementation**: Add validation for file existence and better error messages
   - **Test Cases**: Add test cases for invalid file paths and missing external modules

### Deal with Prior Changes

The documentation has been successfully reorganized according to the new standards:

- **Section 0. Summary**: Added for all four files, explaining their purpose in the context of Assassyn's architecture
- **Section 1. Exposed Interfaces**: Reorganized with proper function signatures and docstrings
- **Section 2. Internal Helpers**: Detailed explanations of all classes, methods, and their purposes
- **Function Documentation**: Each function now includes proper explanations with step-by-step breakdowns
- **Project-Specific Knowledge**: Referenced appropriate design documents for architectural context

All four files (`downstream.md`, `external.md`, `fsm.md`, `module.md`) have been moved from "TO CHECK" to "DONE" in the documentation status.

## Unresolved Issues

1. **Legacy Naming**: The "Module" vs "PipelineStage" naming inconsistency remains in the codebase and should be addressed in a separate refactoring effort.

2. **FSM Debug Output**: The print statements in FSM class should be replaced with proper logging or made configurable.

3. **External Module Enhancement**: The TODO about Verilator integration for Rust simulator should be tracked as a separate feature request.

4. **Missing Test Coverage**: Some edge cases in the module classes may need additional test coverage, particularly around error handling and edge cases.
