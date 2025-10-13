# TODO-doc-fix-simulator-elaboration.md

## Section 1: Goal

Document and clarify unclear aspects and potential inconsistencies found during the documentation of the first four unchecked elements in the DOCUMENTATION-STATUS.md file. This includes addressing project-specific knowledge gaps, naming inconsistencies, and areas that may require human intervention for proper resolution.

## Section 2: Action Items

### Document Development

The following issues were identified during the documentation process and require human intervention:

#### 2.1 Project-Specific Knowledge Gaps

**Issue**: The relationship between Python and Rust implementations is not fully documented.

**Details**: 
- Multiple functions in `codegen/simulator/` reference "This matches the Rust function in src/backend/simulator/elaborate.rs" but the actual Rust implementation is not accessible or documented
- The `elaborate_impl` function claims to match Rust behavior but the verification mechanism is unclear
- The `int_imm_dumper_impl` and `fifo_name` functions reference Rust implementations that are not documented

**Required Action**: 
- Document the relationship between Python and Rust implementations
- Provide clear guidelines on how to verify consistency between implementations
- Create documentation for the Rust backend functions referenced

#### 2.2 Naming Inconsistencies

**Issue**: The term "module" is used inconsistently throughout the codebase.

**Details**:
- The design documents refer to "module" as a pipeline stage, but the code uses "module" for both pipeline stages and downstream modules
- The `ip/multiply.py` file uses "MulStage" classes that are actually modules, not stages
- The documentation mentions "module" as a credited pipeline stage but this naming convention is not consistently applied

**Required Action**:
- Clarify the distinction between "module" and "stage" terminology
- Update naming conventions to be consistent across the codebase
- Provide clear guidelines for when to use "module" vs "stage" terminology

#### 2.3 Undocumented Project-Specific Knowledge

**Issue**: Several implementation details rely on undocumented project-specific knowledge.

**Details**:
- The `multiply.py` file uses specific bit manipulation techniques that are not documented
- The cycle counting logic (cnt-1, cnt < 35) is not explained
- The FIFO naming convention and its relationship to module ports is not fully documented
- The relationship between Assassyn data types and Rust types is not comprehensively documented

**Required Action**:
- Document the bit manipulation techniques used in arithmetic operations
- Explain the cycle counting and timing logic used in pipelined operations
- Document the FIFO naming conventions and their relationship to module ports
- Create comprehensive documentation for data type mapping between Assassyn and Rust

#### 2.4 Code Generation Pipeline Dependencies

**Issue**: The dependencies between different code generation components are not fully documented.

**Details**:
- The relationship between `elaborate.py`, `modules.py`, `simulator.py`, and `node_dumper.py` is not clearly documented
- The order of operations in the code generation pipeline is not specified
- The role of utility functions in the overall pipeline is not documented

**Required Action**:
- Document the code generation pipeline and its dependencies
- Specify the order of operations in the pipeline
- Document the role of each component in the overall pipeline

### Coding Development

#### 2.5 Code Improvements

**Issue**: Several code improvements could enhance maintainability and clarity.

**Details**:
- The `multiply.py` file uses hardcoded values (32, 35) that could be made configurable
- The error handling in `elaborate.py` could be more robust
- The dispatch table in `node_dumper.py` could be more extensible

**Required Action**:
- Make hardcoded values configurable where appropriate
- Improve error handling in critical functions
- Enhance extensibility of dispatch mechanisms

#### 2.6 Documentation Improvements

**Issue**: Several documentation improvements could enhance understanding.

**Details**:
- The relationship between simulator and Verilog generation is not clearly documented
- The role of the port manager in the overall system is not fully explained
- The relationship between Assassyn IR and generated Rust code is not comprehensively documented

**Required Action**:
- Document the relationship between simulator and Verilog generation
- Explain the role of the port manager in the overall system
- Create comprehensive documentation for the IR to Rust code generation process

## Section 3: Impact Assessment

These issues impact the maintainability and understanding of the codebase. Addressing them will:

1. Improve code maintainability by clarifying naming conventions
2. Enhance developer experience by documenting project-specific knowledge
3. Reduce bugs by clarifying implementation details
4. Improve code quality by addressing inconsistencies

## Section 4: Testing Requirements

After addressing these issues, the following tests should be performed:

1. Verify that all documented functions work as described
2. Test the consistency between Python and Rust implementations
3. Validate that naming conventions are consistently applied
4. Ensure that the code generation pipeline works correctly

## Section 5: Dependencies

This TODO depends on:

1. Access to the Rust backend implementation
2. Understanding of the overall system architecture
3. Clarification of naming conventions from the project maintainers
4. Documentation of project-specific knowledge from domain experts
