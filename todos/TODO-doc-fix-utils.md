# Documentation Issues for utils.py

**Date**: 2024-12-19  
**File**: `python/assassyn/utils.py` → `python/assassyn/utils.md`  
**Status**: Documentation reorganized but issues remain

---

## Issues Identified

### 1. Incomplete Implementation - create_and_clean_dir

**Issue**: The function `create_and_clean_dir()` has a misleading name and incomplete implementation.

**Details**:
- Function name suggests it should "clean" (clear contents) of existing directories
- Actual implementation only creates directories using `os.makedirs(dir_path, exist_ok=True)`
- No cleaning/clearing functionality is implemented
- This appears to be an incomplete implementation

**Impact**: Users expecting directory cleaning functionality will be confused by the behavior.

**Recommendation**: Either implement the cleaning functionality or rename the function to `create_dir()` or `ensure_dir()`.

### 2. Inconsistent Default Slice Documentation

**Issue**: Documentation inconsistency regarding the default slice for `identifierize()`.

**Details**:
- Function docstring says "default is slice(-5:-1)"
- Actual implementation uses `Singleton.id_slice` which defaults to `slice(-6, -1)`
- Documentation in `builder.md` correctly states `slice(-6, -1)`

**Impact**: Confusion about the actual default behavior.

**Recommendation**: Update the function docstring to match the actual implementation.

### 3. Missing Project-Specific Knowledge

**Issue**: Several functions lack detailed explanations of their role in the broader project context.

**Details**:
- `patch_fifo()`: Why is FIFO normalization needed? What causes the numbered FIFO instantiations?
- `parse_verilator_cycle()` and `parse_simulator_cycle()`: What is the expected format of the simulation output?
- The relationship between these utilities and the overall simulation workflow is not well documented

**Impact**: New developers may not understand the purpose and context of these functions.

**Recommendation**: Add more detailed explanations about the simulation workflow and why these specific parsing patterns are needed.

### 4. Missing Error Handling Documentation

**Issue**: Functions that interact with external systems lack documentation about error handling.

**Details**:
- `run_simulator()` and `run_verilator()`: What happens if cargo/verilator commands fail?
- `repo_path()`: What happens if `ASSASSYN_HOME` is not set?
- `patch_fifo()`: What happens if the file doesn't exist or is not readable?

**Impact**: Users may not understand error conditions and how to handle them.

**Recommendation**: Document expected error conditions and behavior.

### 5. Global State Management

**Issue**: The use of global `PATH_CACHE` is not well documented.

**Details**:
- Global variable `PATH_CACHE` is used to cache repository path
- No documentation about thread safety or reinitialization scenarios
- No clear lifecycle management

**Impact**: Potential issues in multi-threaded environments or when environment variables change.

**Recommendation**: Document the caching behavior and any potential issues.

---

## Unresolved Questions

1. **FIFO Patching**: Why do numbered FIFO instantiations occur in the first place? Is this a bug in the code generation or an intentional feature?

2. **Simulation Output Format**: What is the exact format of the simulation output that the cycle parsing functions expect?

3. **Environment Dependencies**: What are the exact requirements for `ASSASSYN_HOME` and `VERILATOR_ROOT` environment variables?

4. **Thread Safety**: Are these utility functions safe to use in multi-threaded environments?

---

## Next Steps

1. Investigate the FIFO generation process to understand why patching is needed
2. Examine simulation output examples to document the expected format
3. Test error conditions and document behavior
4. Consider refactoring `create_and_clean_dir()` to match its name or rename it
5. Update function docstrings to match actual implementation
