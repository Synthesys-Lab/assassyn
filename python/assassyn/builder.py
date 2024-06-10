from decorator import decorator
import inspect

class Singleton(type):
    builder = None

@decorator
def ir_builder(func, node_type=None, *args, **kwargs):
    res = func(*args, **kwargs)
    res.id = Singleton.builder.inc_id()
    Singleton.builder.insert_point[node_type].append(res)
    return res

class SysBuilder(object):

    def inc_id(self):
        res = self.cur_module.node_cnt
        self.cur_module.node_cnt += 1
        return res 

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.arrays = []
        self.insert_point = { 'array': self.arrays, 'expr': None, 'module': self.modules }
        self.cur_module = None

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

