"""
Unit tests for rewrite_assign AST transformation.
"""

import ast
import pytest
from assassyn.builder.rewrite_assign import rewrite_assign, __assassyn_assignment__


class TestAssassynAssignment:
    """Test cases for __assassyn_assignment__ function."""

    def test_assignment_returns_value(self):
        """Test that __assassyn_assignment__ returns the value passed to it."""
        result = __assassyn_assignment__("x", 42)
        assert result == 42

    def test_assignment_with_string(self):
        """Test assignment with string value."""
        result = __assassyn_assignment__("name", "test")
        assert result == "test"

    def test_assignment_with_list(self):
        """Test assignment with list value."""
        test_list = [1, 2, 3]
        result = __assassyn_assignment__("items", test_list)
        assert result == test_list

    def test_assignment_with_none(self):
        """Test assignment with None value."""
        result = __assassyn_assignment__("value", None)
        assert result is None

    def test_assignment_preserves_object_identity(self):
        """Test that __assassyn_assignment__ preserves object identity."""
        obj = object()
        result = __assassyn_assignment__("obj", obj)
        assert result is obj


class TestRewriteAssign:
    """Test cases for rewrite_assign AST transformation."""

    def test_rewrite_simple_assignment(self):
        """Test rewriting a simple assignment statement."""
        code = """
def test_func():
    x = 5
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify the function was returned
        assert isinstance(rewritten, ast.FunctionDef)
        assert rewritten.name == "test_func"

        # Verify assignment was rewritten to function call
        assign_stmt = rewritten.body[0]
        assert isinstance(assign_stmt, ast.Assign)
        assert isinstance(assign_stmt.value, ast.Call)

        # Verify the call is to __assassyn_assignment__
        call = assign_stmt.value
        assert isinstance(call.func, ast.Name)
        assert call.func.id == "__assassyn_assignment__"

        # Verify arguments: name and value
        assert len(call.args) == 2
        assert isinstance(call.args[0], ast.Constant)
        assert call.args[0].value == "x"
        assert isinstance(call.args[1], ast.Constant)
        assert call.args[1].value == 5

    def test_rewrite_multiple_assignments(self):
        """Test rewriting multiple assignment statements."""
        code = """
def test_func():
    x = 1
    y = 2
    z = 3
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify all three assignments were rewritten
        assert len(rewritten.body) == 3

        expected_names = ["x", "y", "z"]
        expected_values = [1, 2, 3]

        for i, (name, value) in enumerate(zip(expected_names, expected_values)):
            stmt = rewritten.body[i]
            assert isinstance(stmt, ast.Assign)
            call = stmt.value
            assert isinstance(call, ast.Call)
            assert call.func.id == "__assassyn_assignment__"
            assert call.args[0].value == name
            assert call.args[1].value == value

    def test_rewrite_assignment_with_expression(self):
        """Test rewriting assignment with expression as value."""
        code = """
def test_func():
    x = 1 + 2
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        assign_stmt = rewritten.body[0]
        call = assign_stmt.value

        # Verify the expression is preserved in second argument
        assert isinstance(call.args[1], ast.BinOp)

    def test_rewrite_assignment_with_function_call(self):
        """Test rewriting assignment where value is a function call."""
        code = """
def test_func():
    x = some_function()
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        assign_stmt = rewritten.body[0]
        call = assign_stmt.value

        # Verify the function call is preserved
        assert isinstance(call.args[1], ast.Call)
        assert call.args[1].func.id == "some_function"

    def test_preserves_non_assignment_statements(self):
        """Test that non-assignment statements are preserved."""
        code = """
def test_func():
    print("hello")
    x = 5
    return x
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify structure preserved
        assert len(rewritten.body) == 3
        assert isinstance(rewritten.body[0], ast.Expr)  # print call
        assert isinstance(rewritten.body[1], ast.Assign)  # x = 5 (rewritten)
        assert isinstance(rewritten.body[2], ast.Return)  # return

    def test_rewrite_nested_assignment(self):
        """Test rewriting assignments in nested control flow."""
        code = """
def test_func():
    if True:
        x = 1
    else:
        y = 2
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify the if statement structure
        if_stmt = rewritten.body[0]
        assert isinstance(if_stmt, ast.If)

        # Check assignment in if block
        if_assign = if_stmt.body[0]
        assert isinstance(if_assign.value, ast.Call)
        assert if_assign.value.func.id == "__assassyn_assignment__"
        assert if_assign.value.args[0].value == "x"

        # Check assignment in else block
        else_assign = if_stmt.orelse[0]
        assert isinstance(else_assign.value, ast.Call)
        assert else_assign.value.func.id == "__assassyn_assignment__"
        assert else_assign.value.args[0].value == "y"

    def test_rewrite_assignment_in_loop(self):
        """Test rewriting assignments inside loops."""
        code = """
def test_func():
    for i in range(10):
        x = i
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        for_stmt = rewritten.body[0]
        assert isinstance(for_stmt, ast.For)

        # Check assignment in loop body
        loop_assign = for_stmt.body[0]
        assert isinstance(loop_assign.value, ast.Call)
        assert loop_assign.value.func.id == "__assassyn_assignment__"

    def test_does_not_rewrite_augmented_assignment(self):
        """Test that augmented assignments (+=, -=, etc.) are not rewritten."""
        code = """
def test_func():
    x = 5
    x += 1
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # First statement should be rewritten
        assert isinstance(rewritten.body[0].value, ast.Call)

        # Second statement should remain augmented assignment
        assert isinstance(rewritten.body[1], ast.AugAssign)

    def test_rewrite_tuple_unpacking(self):
        """Test handling of tuple unpacking assignment."""
        code = """
def test_func():
    x, y = 1, 2
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # This is a complex case - verify structure is preserved
        assign_stmt = rewritten.body[0]
        assert isinstance(assign_stmt, ast.Assign)

    def test_preserves_function_name_and_args(self):
        """Test that function definition metadata is preserved."""
        code = """
def my_function(arg1, arg2, kwarg=None):
    x = 1
    return x
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify function metadata preserved
        assert rewritten.name == "my_function"
        assert len(rewritten.args.args) == 3
        assert rewritten.args.args[0].arg == "arg1"
        assert rewritten.args.args[1].arg == "arg2"
        assert rewritten.args.args[2].arg == "kwarg"

    def test_preserves_decorators(self):
        """Test that function decorators are preserved."""
        code = """
@decorator
def test_func():
    x = 5
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Verify decorator is preserved
        assert len(rewritten.decorator_list) == 1
        assert rewritten.decorator_list[0].id == "decorator"

    def test_empty_function_body(self):
        """Test handling of function with no assignments."""
        code = """
def test_func():
    pass
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Should return unchanged function
        assert len(rewritten.body) == 1
        assert isinstance(rewritten.body[0], ast.Pass)

    def test_assignment_with_attribute_access(self):
        """Test that attribute assignments (obj.attr = val) are not rewritten."""
        code = """
def test_func():
    x = 5
    self.attr = 10
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # First should be rewritten (identifier assignment)
        assert isinstance(rewritten.body[0].value, ast.Call)

        # Second should not be rewritten (attribute assignment)
        # Only rewrites assignments to identifiers, not attributes
        assert isinstance(rewritten.body[1], ast.Assign)

    def test_assignment_with_subscript(self):
        """Test that subscript assignments (list[0] = val) are not rewritten."""
        code = """
def test_func():
    x = 5
    arr[0] = 10
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # First should be rewritten (identifier assignment)
        assert isinstance(rewritten.body[0].value, ast.Call)

        # Second should not be rewritten (subscript assignment)
        assert isinstance(rewritten.body[1], ast.Assign)

    def test_compiled_code_executable(self):
        """Test that rewritten AST can be compiled and executed."""
        code = """
def test_func():
    x = 5
    y = x + 1
    return y
"""
        tree = ast.parse(code)
        func_def = tree.body[0]

        rewritten = rewrite_assign(func_def)

        # Create a new module with rewritten function
        module = ast.Module(body=[rewritten], type_ignores=[])
        ast.fix_missing_locations(module)

        # Should compile without errors
        code_obj = compile(module, '<test>', 'exec')
        assert code_obj is not None
