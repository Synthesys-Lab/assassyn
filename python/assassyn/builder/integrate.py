"""
Integration of the naming system with IR builder and combinational decorators.
This module modifies the existing decorators to use the new naming system.
"""

# pylint: disable=duplicate-code

import ast
import functools
import inspect
import textwrap

from .naming_manager import (
    assassyn_assignment_hook,
    set_naming_manager,
    get_naming_manager
)
from .rewrite_assign import rewrite_assign


def ir_builder(func=None, *, node_type=None):
    """
    Enhanced IR builder decorator that integrates with the naming system.

    This replaces the original ir_builder to track created values.
    """

    def _decorate(target):
        @functools.wraps(target)
        def _wrapper(*args, **kwargs):  # pylint: disable=too-many-nested-blocks,too-many-locals
            # Execute the original function
            res = target(*args, **kwargs)

            # If None, nothing to track
            if res is None:
                return res

            # Import required modules
            # pylint: disable=import-outside-toplevel,cyclic-import
            from assassyn.ir.const import Const
            from assassyn.utils import package_path
            from assassyn.ir.expr import Expr
            from assassyn.builder import Singleton
            import os

            # Track the value if we have a naming manager
            manager = get_naming_manager()
            if manager and isinstance(res, Expr):
                manager.push_value(res)

            # Original IR builder logic
            if not isinstance(res, Const):
                if isinstance(res, Expr):
                    res.parent = Singleton.builder.current_block
                    for i in res.operands:
                        Singleton.builder.current_module.add_external(i)
                Singleton.builder.insert_point.append(res)

            # Location tracking
            package_dir = os.path.abspath(package_path())
            Singleton.initialize_dirs_to_exclude()

            for i in inspect.stack()[2:]:  # pylint: disable=too-many-nested-blocks
                fname, lineno = i.filename, i.lineno
                fname_abs = os.path.abspath(fname)

                if not fname_abs.startswith(package_dir) \
                    and not any(
                        fname_abs.startswith(exclude_dir)
                        for exclude_dir in Singleton.all_dirs_to_exclude
                    ):
                    res.loc = f'{fname}:{lineno}'

                    break

            assert hasattr(res, 'loc')
            return res

        if node_type is not None:
            setattr(_wrapper, '_ir_builder_node_type', node_type)
        return _wrapper

    if func is None:
        return _decorate
    return _decorate(func)


def combinational_for(module_class):
    """
    combinational decorator factory that integrates assignment rewriting.
    """

    def combinational(func):
        """
        Decorator for combinational module build functions.

        Rewrites assignments and sets up naming context.
        """

        try:
            # Get the function's AST
            source = textwrap.dedent(inspect.getsource(func))
            tree = ast.parse(source)
            func_def = tree.body[0]

            # Rewrite assignments in the AST
            rewritten_func_def = rewrite_assign(func_def)
            # Remove original decorators to avoid reapplying combinational wrapper
            rewritten_func_def.decorator_list = []

            # Compile the rewritten function
            tree.body[0] = rewritten_func_def
            ast.fix_missing_locations(tree)

            # Reuse the original globals so later definitions remain visible.
            namespace = func.__globals__
            had_assignment_hook = '__assassyn_assignment__' in namespace
            previous_hook = namespace.get('__assassyn_assignment__')
            namespace['__assassyn_assignment__'] = assassyn_assignment_hook

            # Compile and execute to get the new function
            code = compile(tree, func.__code__.co_filename, 'exec')
            exec(code, namespace)  # pylint: disable=exec-used
            new_func = namespace[func.__name__]

            if had_assignment_hook:
                namespace['__assassyn_assignment__'] = previous_hook
        except Exception as e:  # pylint: disable=broad-except
            # If rewriting fails, use original function
            # pylint: disable=import-outside-toplevel
            import sys
            print(f"Warning: AST rewriting failed for {func.__name__}: {e}", file=sys.stderr)
            new_func = func

        @functools.wraps(func)
        def wrapper(self, *args, **kwargs):
            # Import required modules
            # pylint: disable=import-outside-toplevel,cyclic-import
            from assassyn.builder import Singleton
            from assassyn.ir.block import Block

            # Enter module context
            module = self
            assert module is not None

            # Set up naming context
            naming_manager = Singleton.naming_manager
            if naming_manager:
                naming_manager.enable_assignment_hook()

            try:
                # Enter block context
                module.body = Block(Block.MODULE_ROOT)
                module.body.parent = module  # ensure root block tracks owning module
                module.body.module = module
                Singleton.builder.enter_context_of('module', module)
                Singleton.builder.enter_context_of('block', module.body)

                # Execute the rewritten function
                result = new_func(self, *args, **kwargs)

                return result

            finally:
                # Exit contexts
                Singleton.builder.exit_context_of('block')
                Singleton.builder.exit_context_of('module')

                # Disable assignment hook
                if naming_manager:
                    naming_manager.disable_assignment_hook()

        # Mark as combinational
        wrapper._is_combinational = True  # pylint: disable=protected-access
        wrapper._module_class = module_class  # pylint: disable=protected-access
        wrapper.__assassyn_original__ = new_func

        return wrapper

    return combinational


def install_decorators():
    """
    Install the enhanced decorators in place of the original ones.

    This should be called at system initialization.
    """
    # pylint: disable=import-outside-toplevel,cyclic-import
    import assassyn.builder as builder_module
    import assassyn.ir.module as module_package
    import assassyn.ir.module.base as base_module
    from assassyn.ir.module import module as module_module
    from assassyn.ir.module import downstream as downstream_module

    # Replace ir_builder
    builder_module.ir_builder = ir_builder

    # Replace combinational_for factory
    base_module.combinational_for = combinational_for

    # Update package-level decorators so downstream imports pick up the new behavior
    module_combinational = combinational_for(module_package.Module)
    downstream_combinational = combinational_for(module_package.Downstream)

    module_package.combinational = module_combinational
    module_package.downstream_combinational = downstream_combinational

    # Update existing combinational decorators exposed from submodules
    module_module.combinational = module_combinational
    downstream_module.combinational = downstream_combinational


def sys_builder():
    """
    Enhance the SysBuilder class to properly initialize naming.

    This modifies the __enter__ and __exit__ methods.
    """
    # pylint: disable=import-outside-toplevel,cyclic-import
    from assassyn.builder import SysBuilder

    # Store original methods
    original_enter = SysBuilder.__enter__
    original_exit = SysBuilder.__exit__

    def new_enter(self):
        """Enhanced __enter__ that sets up naming manager."""
        # Call original
        result = original_enter(self)

        # Set global naming manager
        set_naming_manager(self.naming_manager)

        return result

    def new_exit(self, exc_type, exc_value, traceback):
        """Enhanced __exit__ that clears naming manager."""
        # Clear global naming manager
        set_naming_manager(None)

        # Call original
        return original_exit(self, exc_type, exc_value, traceback)

    # Replace methods
    SysBuilder.__enter__ = new_enter
    SysBuilder.__exit__ = new_exit


# ============================================================================
# Auto-initialization
# ============================================================================

def initialize_naming_system():
    """
    Initialize the entire naming system.

    Call this once at module load or system start.
    """
    install_decorators()
    sys_builder()
