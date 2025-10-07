"""
AST transformer to rewrite assignment statements.

This module provides functionality to rewrite Python assignment statements
to use a custom assignment function that can be hooked for tracing or other purposes.
"""

import ast
from typing import Any
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
