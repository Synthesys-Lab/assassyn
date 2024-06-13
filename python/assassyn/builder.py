from decorator import decorator
import inspect
import types

class Singleton(type):
    builder = None
    linearize_ptr = None
    repr_ident = None

@decorator
def ir_builder(func, node_type=None, *args, **kwargs):
    res = func(*args, **kwargs)
    module_symtab = Singleton.builder.is_direct_call(inspect.currentframe())
    Singleton.builder.insert_point[node_type].append(res)
    # This is to have a symbol table for the module currently being built,
    # so that we can name those named expressions.
    if module_symtab is not None:
        Singleton.builder.named_expr.append(res)
        Singleton.builder.module_symtab = module_symtab
    return res

class SysBuilder(object):

    def cleanup_symtab(self):
        value_dict = { id(v): v for v in self.named_expr }
        for k, v in self.module_symtab.items():
            if id(v) in value_dict:
                value_dict[id(v)].name = k

    def is_direct_call(self, frame: types.FrameType):
        upper_frame = frame.f_back.f_back
        if not upper_frame.f_locals.get('self') is self.cur_module:
            return None
        upper_frame = upper_frame.f_back
        caller = upper_frame.f_code.co_name
        if caller == 'combinational':
            return frame.f_back.f_back.f_locals
        return None

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.arrays = []
        self.insert_point = { 'array': self.arrays, 'expr': None, 'module': self.modules }
        self.cur_module = None
        self.builder_func = None
        self.module_symtab = None
        self.named_expr = []

    def __enter__(self):
        assert Singleton.builder is None
        Singleton.builder = self
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        assert Singleton.builder is self
        Singleton.builder = None

    def __repr__(self):
        body = '\n\n'.join(map(repr, self.modules))
        array = '  ' + '\n  '.join(repr(elem) for elem in self.arrays)
        return f'system {self.name} {{\n{array}\n\n{body}\n}}'

