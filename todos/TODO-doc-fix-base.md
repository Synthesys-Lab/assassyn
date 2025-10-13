# TODO: Documentation Fix for Base Module

## Section 1: Goal

Complete the documentation review and fix for `ir/module/base.py` by addressing unclear aspects and potential inconsistencies found during analysis.

## Section 2: Action Items

### Document Development

1. **Clarify ModuleBase.triggered() Usage Context**
   - **Issue**: The `triggered()` method documentation states it's "only usable in downstream modules" but the implementation doesn't enforce this restriction.
   - **Analysis**: The method creates a `PureIntrinsic` with `MODULE_TRIGGERED` opcode, but there's no runtime check preventing its use in regular modules.
   - **Action Required**: Either add runtime validation to enforce the restriction, or update documentation to clarify the actual usage patterns and limitations.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`, `python/assassyn/ir/module/base.md`

2. **Document External Dependency Tracking Semantics**
   - **Issue**: The `add_external()` method's logic for determining "external" dependencies is complex and not fully documented.
   - **Analysis**: The method checks if an operand references `Array`, `Module`, or expressions from different modules, but the exact criteria and edge cases are unclear.
   - **Action Required**: Add detailed documentation explaining the external dependency detection logic, including examples of what constitutes an external dependency.
   - **Files to Modify**: `python/assassyn/ir/module/base.md`

3. **Clarify combinational_for Decorator Parameter Binding**
   - **Issue**: The decorator's parameter binding logic for `Array` objects is complex and has specific rules about hierarchical naming that aren't documented.
   - **Analysis**: The code preserves hierarchical names (with underscores, except 'array_xxxxx' pattern) but this logic is not explained.
   - **Action Required**: Document the parameter binding rules, especially the hierarchical naming preservation logic for `Array` objects.
   - **Files to Modify**: `python/assassyn/ir/module/base.md`

### Coding Development

4. **Add Runtime Validation for triggered() Method**
   - **Issue**: The `triggered()` method should only be usable in downstream modules but lacks runtime enforcement.
   - **Action Required**: Add runtime check in `triggered()` method to ensure it's only called from downstream modules.
   - **Test Case**: Create test case in `python/ci-tests/` to verify the restriction works correctly.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`, `python/ci-tests/test_base_validation.py` (new)

5. **Improve Error Handling in combinational_for**
   - **Issue**: The decorator handles AST rewriting failures gracefully but doesn't provide detailed error information.
   - **Action Required**: Enhance error reporting to provide more specific information about AST rewriting failures.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`

### Deal with Prior Changes

6. **Cross-reference with Related Documentation**
   - **Issue**: The base module documentation should better reference related modules and their relationships.
   - **Action Required**: Add cross-references to `ir/module/module.md`, `ir/module/downstream.md`, and `builder/rewrite_assign.md` to clarify the relationships.
   - **Files to Modify**: `python/assassyn/ir/module/base.md`

## Section 3: Unclear Aspects Requiring Human Intervention

1. **External Dependency Detection Edge Cases**
   - The current logic for detecting external dependencies may have edge cases with nested expressions or complex operand chains that need human review.
   - **Question**: Are there specific patterns of external dependencies that should be explicitly supported or avoided?

2. **AST Rewriting Failure Scenarios**
   - The `combinational_for` decorator falls back to the original function when AST rewriting fails, but the specific failure scenarios aren't documented.
   - **Question**: What are the common causes of AST rewriting failures, and should they be handled differently?

3. **ModuleBase Inheritance Hierarchy**
   - The relationship between `ModuleBase` and its derived classes (`Module`, `Downstream`) could be better documented.
   - **Question**: Should `ModuleBase` be considered an abstract base class, and are there any methods that should be abstract?

## Section 4: Potential Code Improvements

1. **Add Type Hints**
   - The `combinational_for` decorator factory could benefit from better type hints for the returned decorator.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`

2. **Improve Error Messages**
   - The `add_external()` method could provide more informative error messages when external dependency detection fails.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`

3. **Add Validation Methods**
   - Consider adding validation methods to `ModuleBase` to check module state consistency.
   - **Files to Modify**: `python/assassyn/ir/module/base.py`
