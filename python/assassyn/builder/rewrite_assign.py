"""
AST transformer to rewrite assignment statements.

This module provides functionality to rewrite Python assignment statements
to use a custom assignment function that can be hooked for tracing or other purposes.
"""

import ast
from typing import Any
import inspect
import textwrap
from functools import wraps


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
    # Import here to avoid circular import
    from . import Singleton  # pylint: disable=import-outside-toplevel,cyclic-import
    manager = Singleton.builder.naming_manager if Singleton.builder else None
    if manager:
        return manager.process_assignment(name, value)
    return value


class AssignmentRewriter(ast.NodeTransformer):
    """AST transformer that rewrites assignments to identifiers."""
    def visit_Assign(self, node: ast.Assign) -> ast.Assign:  # pylint: disable=invalid-name
        """Rewrite simple identifier assignments to use __assassyn_assignment__."""
        self.generic_visit(node)
        if len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            target = node.targets[0]
            new_value = ast.Call(
                func=ast.Name(id="__assassyn_assignment__", ctx=ast.Load()),
                args=[ast.Constant(value=target.id), node.value],
                keywords=[]
            )
            return ast.Assign(targets=node.targets, value=new_value)
        return node

def rewrite_assign(func=None, *, adjust_lineno=False):
    """
    Decorator to rewrite assignment statements in a function to use __assassyn_assignment__.

    This decorator can be used in two ways:
    1. As a simple decorator: @rewrite_assign
    2. With parameters: @rewrite_assign(adjust_lineno=True)

    The decorator parses the function's source, transforms assignments to use
    __assassyn_assignment__, and returns the rewritten function.

    Args:
        func: The function to rewrite (when used as @rewrite_assign)
        adjust_lineno: If True, adjust AST line numbers to match original source location

    Returns:
        The rewritten function (or decorator if called with parameters)
    """
    def _decorator(target_func):  # pylint: disable=too-many-locals
        try:
            # Parse source and get AST
            source = textwrap.dedent(inspect.getsource(target_func))
            original_lineno = target_func.__code__.co_firstlineno

            tree = ast.parse(source)
            func_def = tree.body[0]

            # Rewrite assignments
            rewriter = AssignmentRewriter()
            rewritten_func_def = rewriter.visit(func_def)
            rewritten_func_def.decorator_list = []

            tree.body[0] = rewritten_func_def
            ast.fix_missing_locations(tree)

            # Adjust line numbers if requested
            if adjust_lineno:
                line_offset = original_lineno - 1
                for node in ast.walk(tree):
                    if hasattr(node, 'lineno'):
                        node.lineno += line_offset
                    if hasattr(node, 'end_lineno') and node.end_lineno is not None:
                        node.end_lineno += line_offset

            # Inject assignment hook and compile
            namespace = target_func.__globals__
            had_assignment_hook = '__assassyn_assignment__' in namespace
            previous_hook = namespace.get('__assassyn_assignment__')
            namespace['__assassyn_assignment__'] = __assassyn_assignment__

            code = compile(tree, target_func.__code__.co_filename, 'exec')
            exec(code, namespace)  # pylint: disable=exec-used
            new_func = namespace[target_func.__name__]

            # Restore previous hook if it existed
            if had_assignment_hook:
                namespace['__assassyn_assignment__'] = previous_hook

            # Preserve function metadata
            new_func = wraps(target_func)(new_func)

            return new_func

        except Exception as exc:  # pylint: disable=broad-except
            # Fallback to original function if rewriting fails
            import sys  # pylint: disable=import-outside-toplevel
            print(f"Warning: AST rewriting failed for {target_func.__name__}: {exc}",
                  file=sys.stderr)
            return target_func

    # Handle both @rewrite_assign and @rewrite_assign(adjust_lineno=True)
    if func is None:
        return _decorator
    return _decorator(func)
