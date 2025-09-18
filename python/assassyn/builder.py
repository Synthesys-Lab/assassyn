'''The module provides the implementation of a class that is both IR builder and the system.'''

#pylint: disable=cyclic-import

from __future__ import annotations
import os
import typing
import site
import inspect
import ast
from decorator import decorator
from .namify import NamingManager

if typing.TYPE_CHECKING:
    from .ir.module import Module
    from .ir.array import Array
    from .ir.dtype import DType
    from .ir.value import Value

def process_naming(expr, line_of_code: str, lineno: int) -> typing.Dict[str, typing.Any]:
    """Process naming for an expression based on line context"""

    line_expression_tracker = Singleton.line_expression_tracker
    naming_manager = Singleton.naming_manager
    try:
        parsed_ast = ast.parse(line_of_code)
        # print(ast.dump(parsed_ast, indent=4))
        if parsed_ast.body and isinstance(parsed_ast.body[0], ast.Assign):
            assign_node = parsed_ast.body[0]

            if lineno not in line_expression_tracker:
                line_expression_tracker[lineno] = {
                    'expressions': [],
                    'assign_node': assign_node,
                    'names_generated': False,
                    'generated_names': []
                }
            line_data = line_expression_tracker[lineno]

            if  line_data['names_generated'] and expr.opcode == 800:
                line_data = line_expression_tracker[lineno]
                expr_position = len(line_data['expressions']) - 1
                generated_names = line_data['generated_names']
                if expr_position < len(generated_names):
                    # Ensure uniqueness for cast operations
                    base_name = f"{generated_names[expr_position]}_cast"
                    # Use the naming manager to ensure global uniqueness
                    unique_name = naming_manager.strategy.get_unique_name(base_name)
                    return unique_name

            line_data['expressions'].append(expr)

            if not line_data['names_generated']:
                generated_names = naming_manager.generate_source_names(
                    lineno, assign_node
                )
                line_data['generated_names'] = generated_names
                line_data['names_generated'] = True

            expr_position = len(line_data['expressions']) - 1
            generated_names = line_data['generated_names']

            if expr_position < len(generated_names):
                source_name = generated_names[expr_position]
            else:
                base_name = generated_names[0] if generated_names else "expr"
                source_name = f"tmp_{base_name}_{expr_position}"
                source_name = naming_manager.strategy.get_unique_name(source_name)

            return source_name

    except SyntaxError:
        pass


    return None


@decorator
def ir_builder(func, *args, **kwargs):
    '''The decorator annotates the function whose return value will be inserted into the AST.'''
    res = func(*args, **kwargs)

    # This indicates this res is handled somewhere else, so we do not need to rehandle it
    if res is None:
        return res

    #pylint: disable=cyclic-import,import-outside-toplevel
    from .ir.const import Const
    from .utils import package_path
    from .ir.expr import Expr

    if not isinstance(res, Const):
        if isinstance(res, Expr):
            res.parent = Singleton.builder.current_block
            for i in res.operands:
                Singleton.builder.current_module.add_external(i)
        Singleton.builder.insert_point.append(res)

    package_dir = os.path.abspath(package_path())

    Singleton.initialize_dirs_to_exclude()
    for i in inspect.stack()[2:]:
        fname, lineno = i.filename, i.lineno
        fname_abs = os.path.abspath(fname)

        if not fname_abs.startswith(package_dir) \
            and not any(fname_abs.startswith(exclude_dir) \
                         for exclude_dir in Singleton.all_dirs_to_exclude):
            res.loc = f'{fname}:{lineno}'

            if isinstance(res, Expr):
                if res.is_valued() and i.code_context:
                    line_of_code = i.code_context[0].strip()

                    naming_result = process_naming(
                    res, line_of_code, lineno
                )
                    if naming_result:
                        res.source_name = naming_result
            break
    assert hasattr(res, 'loc')
    return res


#pylint: disable=too-many-instance-attributes
class SysBuilder:
    '''The class serves as both the system and the IR builder.'''

    name: str  # Name of the system
    modules: typing.List[Module]  # List of modules
    downstreams: list  # List of downstream modules
    arrays: typing.List[Array]  # List of arrays
    _ctx_stack: dict  # Stack for context tracking
    _exposes: dict  # Dictionary of exposed nodes
    line_expression_tracker: dict  # Dictionary of line expression tracker
    naming_manager: NamingManager  # Naming manager

    @property
    def current_module(self):
        '''Get the current module being built.'''
        return None if not self._ctx_stack['module'] else self._ctx_stack['module'][-1]

    @property
    def current_block(self):
        '''Get the current block being built.'''
        return None if not self._ctx_stack['block'] else self._ctx_stack['block'][-1]

    @property
    def insert_point(self):
        '''Get the insert point.'''
        return self.current_block.body

    def enter_context_of(self, ty, entry):
        '''Enter the context of the given type.'''
        #pylint: disable=import-outside-toplevel
        from .ir.block import CondBlock
        if isinstance(entry, CondBlock):
            self.current_module.add_external(entry.cond)
        self._ctx_stack[ty].append(entry)

    def exit_context_of(self, ty):
        '''Exit the context of the given type.'''
        self._ctx_stack[ty].pop()

    def has_driver(self):
        '''Check if the system has a driver module.'''
        for i in self.modules:
            if i.__class__.__name__ == 'Driver':
                return True
        return False

    def has_module(self, name):
        '''Check if a module with the given name exists.'''
        for i in self.modules:
            if i.name == name:
                return i
        return None

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.downstreams = []
        self.arrays = []
        self._ctx_stack = {'module': [], 'block': []}
        self._exposes = {}
        self.line_expression_tracker = {}
        self.naming_manager = NamingManager()

    def expose_on_top(self, node, kind=None):
        '''Expose the given node in the top function with the given kind.'''
        self._exposes[node] = kind

    @property
    def exposed_nodes(self):
        '''Get the exposed nodes.'''
        return self._exposes

    def __enter__(self):
        '''Designate the scope of this system builder.'''
        assert Singleton.builder is None
        Singleton.builder = self
        Singleton.line_expression_tracker = self.line_expression_tracker
        Singleton.naming_manager = self.naming_manager
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        '''Leave the scope of this system builder.'''
        assert Singleton.builder is self
        Singleton.builder = None
        Singleton.line_expression_tracker = None
        Singleton.naming_manager = None

    def __repr__(self):
        body = '\n\n'.join(map(repr, self.modules))
        body = body + '\n\n' + '\n\n'.join(map(repr, self.downstreams))
        array = '  ' + '\n  '.join(repr(elem) for elem in self.arrays)
        return f'system {self.name} {{\n{array}\n\n{body}\n}}'

class Singleton(type):
    '''The class maintains the global singleton instance of the system builder.'''
    builder: SysBuilder = None  # Global singleton instance of the system builder
    repr_ident: int = None  # Indentation level for string representation
    id_slice: slice = slice(-6, -1)  # Slice for identifiers
    with_py_loc: bool = False  # Whether to include Python location in string representation
    all_dirs_to_exclude: list = []  # Directories to exclude for stack inspection

    @classmethod
    def initialize_dirs_to_exclude(mcs):
        '''Initialize the directories to exclude if not already initialized.'''
        if not mcs.all_dirs_to_exclude:
            site_package_dirs = site.getsitepackages()
            user_site_package_dir = site.getusersitepackages()
            mcs.all_dirs_to_exclude = site_package_dirs + [user_site_package_dir]
