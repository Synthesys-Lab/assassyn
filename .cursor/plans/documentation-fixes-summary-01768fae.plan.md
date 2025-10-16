<!-- 01768fae-c434-4199-b1b6-2467f5098b70 88ad2ae8-a5f5-47e1-b169-11e6edadc6c2 -->
# Documentation Fixes and Improvements Plan

## Overview

After analyzing all TODO-doc-fix markdown files, several common patterns emerge that can be addressed systematically. Commit 5beeaf35d already resolved several issues in `utils.py` including renaming `create_and_clean_dir` to `create_dir`, improving error handling, and fixing docstring inconsistencies.

## Common Patterns Identified

### 1. Documentation Structure Standardization (MOSTLY RESOLVED)

All documentation files need to follow the new standard with:

- **Section 0: Summary** - High-level purpose and role
- **Section 1: Exposed Interfaces** - Public functions with signatures
- **Section 2: Internal Helpers** - Private methods and implementation details

**Status**: Most files have been reorganized. Remaining work focuses on content improvements.

### 2. Error Handling Documentation (PARTIALLY RESOLVED)

**Pattern**: Multiple modules lack comprehensive error handling documentation

- What exceptions can be raised
- Error recovery strategies
- Edge case behavior

**Already Resolved** (commit 5beeaf35d):

- `utils.repo_path()` - Added comprehensive error handling
- `utils.create_dir()` - Improved error handling with type hints

**Still Needs Work**:

- `codegen/simulator/simulator.py` - No validation of configuration parameters
- `codegen/verilog/design.py` - Limited error handling in complex state management
- `ramulator2/ramulator2.py` - Conditional cleanup logic not fully documented
- `experimental/frontend/` modules - Error handling in factory pattern

### 3. Function Naming Inconsistencies (SOME RESOLVED)

**Pattern**: Function names don't clearly reflect actual behavior

**Already Resolved** (commit 5beeaf35d):

- `create_and_clean_dir()` → `create_dir()` - Name now matches behavior

**Still Needs Work**:

- `cleanup_post_generation()` - Name suggests cleanup but generates signal routing
- `generate_sram_blackbox_files()` - Generates both blackbox and regular modules
- `dump_rval()` - Name suggests dumping but generates signal references
- `topological_sort()` in `analysis/topo.py` - Unused function, inconsistent naming

### 4. Type Hints and Documentation Mismatches (PARTIALLY RESOLVED)

**Pattern**: Documentation doesn't match implementation

**Already Resolved** (commit 5beeaf35d):

- `identifierize()` - Fixed docstring to show correct default slice (-6:-1)
- Global caches - Added type hints (`PATH_CACHE: str | None`)

**Still Needs Work**:

- `Slice` class - Claims `int` return but returns `UInt`
- DRAM parameter order inconsistency - Documentation vs implementation mismatch
- `elaborate()` function - Could use more specific return type hints

### 5. Project-Specific Knowledge Gaps

**Pattern**: Missing context about how components integrate with broader system

**Common Issues**:

- Credit-based pipeline architecture integration not always clear
- Module vs PipelineStage terminology confusion (legacy naming)
- FIFO patching rationale not explained
- Memory system integration (DRAM + Ramulator2) needs clearer docs
- Port mapper and multi-port array architecture

### 6. Global State Management (PARTIALLY RESOLVED)

**Pattern**: Global singletons and state management not well documented

**Already Resolved** (commit 5beeaf35d):

- `PATH_CACHE` and `VERILATOR_CACHE` - Added type hints and documentation

**Still Needs Work**:

- `port_mapper.py` - Global singleton pattern lacks thread safety discussion
- `naming_manager.py` - Global state lifecycle needs better docs
- Block context management - String representation uses global state

### 7. Cross-Module Dependencies

**Pattern**: Relationships between modules not clearly documented

**Common Issues**:

- Expression modules reference each other inconsistently
- Code generation pipeline dependencies unclear
- Python-Rust consistency not well documented
- AST rewriting system integration needs clarification

### 8. Incomplete Implementations and TODOs

**Pattern**: Code has TODOs or incomplete features

**Examples**:

- `Record.attributize()` - Incomplete implementation with TODO comments
- `BinaryOp.dtype` - TODO for carry bit handling
- ExternalSV - TODO about Verilator integration for Rust simulator
- DRAM callback port assignment - Documented but not implemented

## Key Files Requiring Attention

### High Priority

1. **`codegen/simulator/simulator.py`** - Refactor large `dump_simulator` function, add validation
2. **`codegen/verilog/design.py`** - Complex state management in `CIRCTDumper` class
3. **`analysis/topo.py`** - Remove unused `topological_sort()` function or integrate it
4. **Memory modules** - Clarify DRAM parameter order and initialization file format

### Medium Priority

5. **`experimental/frontend/` modules** - Clarify Module vs PipelineStage terminology
6. **`ir/expr/` modules** - Fix type documentation mismatches
7. **Port mapper** - Add thread safety discussion and test coverage
8. **Block module** - Consider enum for block kinds instead of magic numbers

### Low Priority

9. **Various modules** - Extract magic numbers to constants
10. **Code organization** - Break down large functions

## Resolved by Commit 5beeaf35d

✅ `create_and_clean_dir()` renamed to `create_dir()`
✅ `repo_path()` error handling improved
✅ `package_path()` uses `os.path.join()` with type hints
✅ Global cache type hints added
✅ `identifierize()` docstring corrected
✅ All code references updated

## Recommended Actions

### Phase 1: Documentation Corrections (No Code Changes)

- Fix type documentation mismatches
- Document error conditions comprehensively
- Add project-specific knowledge sections
- Clarify cross-module dependencies

### Phase 2: Code Quality Improvements

- Add validation to configuration parameters
- Improve error handling with specific exceptions
- Add type hints where missing
- Extract magic numbers to constants

### Phase 3: Refactoring (Breaking Changes)

- Consider renaming Module → PipelineStage (large impact)
- Remove unused functions (e.g., `topological_sort`)
- Refactor large functions into smaller units
- Consider enum for block kinds and other magic numbers

## Files Summary

**Total TODO Files Analyzed**: 22
**Already Resolved Issues**: ~6 (from TODO-doc-fix-utils.md and TODO-doc-fix-frontend-backend.md)
**Remaining Issues**: ~50+ across all files

## Success Criteria

- All documentation follows new standards structure
- Error conditions documented for all public functions
- Type hints match actual implementation
- Project-specific knowledge gaps filled
- Cross-module relationships clearly documented
- No misleading function names
- Global state management well documented

## Implementation Tasks

The work will be organized into phases, with each phase containing specific actionable tasks.

### Final Step: Stage and Commit Changes

After completing the documentation analysis and creating this comprehensive plan:

1. Stage this plan document
2. Commit with message following project conventions: `docs: add comprehensive documentation fixes summary and action plan`
3. Reference the relevant TODO files and commit 5beeaf35d