"""
AST transformer to rewrite assignment statements.

This module provides functionality to rewrite Python assignment statements
to use a custom assignment function that can be hooked for tracing or other purposes.
"""

import ast
from typing import Any
import inspect
import textwrap
from .naming_manager import get_naming_manager  # pylint: disable=cyclic-import,import-outside-toplevel


def __assassyn_assignment__(name: str, value: Any) -> Any:
    """
    Assignment function invoked by rewritten assignments.

    Delegates to the active NamingManager (if any) to process assignment-based
    naming, then returns the value. When no manager is active, it simply
    returns the value unchanged.

    Args:
        name: Identifier name being assigned to
        value: The value being assigned

    Returns:
        The assigned value (to support chained assignments)
    """
    manager = get_naming_manager()
    if manager:
        return manager.process_assignment(name, value)
    return value


class AssignmentRewriter(ast.NodeTransformer):
    """AST transformer that rewrites assignments to identifiers."""

    def visit_Assign(self, node: ast.Assign) -> ast.Assign:  # pylint: disable=invalid-name
        """
        Visit an assignment node and rewrite it if it's a simple identifier assignment.

        Only rewrites assignments to simple identifiers (Name nodes), not attributes
        or subscripts.
        """
        # Visit child nodes first
        self.generic_visit(node)

        # Only rewrite if target is a single Name (identifier)
        if len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            target = node.targets[0]

            # Create the rewritten assignment: target = __assassyn_assignment__("target", value)
            new_value = ast.Call(
                func=ast.Name(id="__assassyn_assignment__", ctx=ast.Load()),
                args=[
                    ast.Constant(value=target.id),  # The identifier name as a string
                    node.value  # The original value expression
                ],
                keywords=[]
            )

            # Return the modified assignment
            return ast.Assign(targets=node.targets, value=new_value)

        # Return unchanged for non-identifier assignments (attributes, subscripts, tuple unpacking)
        return node


def rewrite_assign(target: ast.FunctionDef) -> ast.FunctionDef:
    """
    Rewrite assignment statements in a function to use __assassyn_assignment__.

    This function takes a function definition AST node and transforms all
    simple identifier assignments (e.g., x = 5) into calls to __assassyn_assignment__
    (e.g., x = __assassyn_assignment__("x", 5)).

    Assignments to attributes (obj.attr = val) and subscripts (arr[i] = val)
    are not rewritten.

    Args:
        target: The function definition AST node to transform

    Returns:
        The transformed function definition AST node
    """
    rewriter = AssignmentRewriter()
    return rewriter.visit(target)


def parse_and_rewrite_function(func, adjust_lineno: bool = False):
    """
    Parse a function's source and rewrite assignments with AST transformation.

    This is a helper function that extracts the common pattern of:
    1. Getting function source code
    2. Parsing it into an AST
    3. Rewriting assignments
    4. Removing decorator list
    5. Optionally adjusting line numbers to match original source location

    Args:
        func: The function to parse and rewrite
        adjust_lineno: If True, adjust AST line numbers to match original source location

    Returns:
        A tuple of (rewritten_tree, original_lineno) where:
        - rewritten_tree: The AST tree with rewritten function
        - original_lineno: The original line number of the function (for reference)
    """
    source = textwrap.dedent(inspect.getsource(func))
    original_lineno = func.__code__.co_firstlineno

    tree = ast.parse(source)
    func_def = tree.body[0]

    rewritten_func_def = rewrite_assign(func_def)
    rewritten_func_def.decorator_list = []

    tree.body[0] = rewritten_func_def
    ast.fix_missing_locations(tree)

    # Adjust all line numbers in the AST to match original source location
    if adjust_lineno:
        line_offset = original_lineno - 1
        for node in ast.walk(tree):
            if hasattr(node, 'lineno'):
                node.lineno += line_offset
            if hasattr(node, 'end_lineno') and node.end_lineno is not None:
                node.end_lineno += line_offset

    return tree, original_lineno


def named(func):
    """
    Decorator to enable semantic naming for assignments in standalone functions.
    """
    try:
        tree, _ = parse_and_rewrite_function(func, adjust_lineno=False)

        namespace = func.__globals__
        namespace['__assassyn_assignment__'] = __assassyn_assignment__

        # Compile and execute to get the rewritten function
        code = compile(tree, func.__code__.co_filename, 'exec')
        exec(code, namespace)  # pylint: disable=exec-used
        new_func = namespace[func.__name__]

        # Preserve function metadata
        new_func.__name__ = func.__name__
        new_func.__doc__ = func.__doc__
        new_func.__module__ = func.__module__
        new_func.__qualname__ = func.__qualname__

        return new_func

    except Exception as exc:  # pylint: disable=broad-except
        # Fallback to original function if rewriting fails
        import sys  # pylint: disable=import-outside-toplevel
        print(f"Warning: AST rewriting failed for {func.__name__}: {exc}", file=sys.stderr)
        return func
