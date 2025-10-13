# TODO: Fix Experimental Frontend Documentation Issues

## Goal

Address documentation inconsistencies and unclear aspects discovered during the review
of the experimental frontend documentation files (`downstream.md`, `factory.md`, `module.md`)
to ensure they fully comply with the new documentation standards.

## Action Items

### Documentation Development

1. **Clarify Module vs Pipeline Stage Terminology**
   - **Issue**: The documentation refers to "Module" as pipeline stages, but the design documents
     consistently use "pipeline stage" terminology. This creates confusion about whether
     "Module" is the correct term or if it's a legacy naming issue.
   - **Required Action**: 
     - Review all design documents to confirm the correct terminology
     - Update documentation to consistently use "pipeline stage" when referring to
       the architectural concept
     - Add a note explaining that "Module" is the implementation class name for
       pipeline stages (legacy naming issue)
   - **Files to Update**: `module.md`, `factory.md`, `downstream.md`

2. **Document Callback Module Type**
   - **Issue**: The factory documentation mentions `Callback` as a supported module type,
     but there is no corresponding `callback.py` or `callback.md` file in the codebase.
     The README also has an RFC question about whether `Callback` should exist.
   - **Required Action**:
     - Determine if `Callback` module type is implemented or planned
     - If implemented, create missing documentation files
     - If not implemented, remove references from documentation
     - Resolve the RFC question in the README
   - **Files to Investigate**: `factory.py`, `factory.md`, `README.md`

3. **Clarify Factory Pattern Implementation Details**
   - **Issue**: The factory pattern implementation has several complex internal mechanisms
     that are not fully documented, particularly around the singleton pattern usage
     and the relationship between the builder context and module construction.
   - **Required Action**:
     - Document the singleton pattern usage in the factory system
     - Explain the builder context management and its relationship to module construction
     - Clarify the AST construction process and how it integrates with the factory pattern
   - **Files to Update**: `factory.md`

4. **Document Port vs Pin Distinction**
   - **Issue**: The documentation uses "port" and "pin" terminology inconsistently.
     Ports are used for sequential communication between pipeline stages, while pins
     are used for combinational communication. This distinction needs to be clearly
     documented.
   - **Required Action**:
     - Add clear definitions distinguishing ports from pins
     - Update all documentation to use consistent terminology
     - Explain when to use ports vs pins in different architectural contexts
   - **Files to Update**: `module.md`, `downstream.md`, `factory.md`

5. **Clarify Timing Modes (Systolic vs Backpressure)**
   - **Issue**: The `pop_all` function documentation mentions systolic and backpressure
     timing modes, but these concepts are not explained in the context of the experimental
     frontend or how they relate to the credit-based pipeline architecture.
   - **Required Action**:
     - Document the timing modes and their relationship to the credit system
     - Explain when each timing mode is used and why
     - Clarify how timing modes affect module behavior
   - **Files to Update**: `module.md`

### Coding Development

6. **Fix Documentation Inconsistencies**
   - **Issue**: Several function signatures in the documentation don't match the actual
     implementation in the source code.
   - **Required Action**:
     - Update `factory_create` signature in `downstream.md` to match implementation
     - Fix parameter names and types to match actual function signatures
     - Ensure all documented function signatures are accurate
   - **Files to Update**: `downstream.md`, `module.md`

7. **Add Missing Usage Examples**
   - **Issue**: The documentation lacks comprehensive usage examples that demonstrate
     the integration between different module types and the factory pattern.
   - **Required Action**:
     - Add examples showing Module-to-Module communication
     - Add examples showing Module-to-Downstream communication
     - Add examples showing complex factory compositions
     - Ensure examples are complete and runnable
   - **Files to Update**: `module.md`, `downstream.md`, `factory.md`

8. **Document Error Handling**
   - **Issue**: The documentation doesn't comprehensively cover error conditions and
     exception handling in the factory system.
   - **Required Action**:
     - Document all possible exceptions that can be raised
     - Explain error recovery strategies
     - Add examples of common error scenarios
   - **Files to Update**: `factory.md`, `module.md`, `downstream.md`

### Testing

9. **Validate Documentation Against Tests**
   - **Required Action**:
     - Review all experimental frontend test files to ensure documentation
       examples match actual usage patterns
     - Update documentation if test patterns reveal different usage than documented
     - Ensure all documented features have corresponding test coverage
   - **Files to Review**: `test_exp_fe_*.py` files in `ci-tests/`

10. **Cross-reference with Design Documents**
    - **Required Action**:
      - Ensure all architectural concepts mentioned in the experimental frontend
        documentation are properly cross-referenced with design documents
      - Verify that implementation details align with architectural decisions
      - Update design documents if experimental frontend reveals new architectural
        patterns not documented elsewhere
    - **Files to Review**: All files in `docs/design/`

## Notes

- The experimental frontend represents a significant architectural evolution from the
  legacy frontend, providing functional programming patterns for module construction
- The factory pattern enables better composability and type safety compared to
  imperative construction patterns
- The credit-based pipeline architecture is fundamental to understanding how modules
  communicate and execute
- Documentation should emphasize the architectural concepts while explaining
  implementation details
