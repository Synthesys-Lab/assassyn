from decorator import decorator

class Singleton(type):
    builder = None

@decorator
def ir_builder(func, node_type='', *args):
    assert(node_type)
    res = func(*args)
    Singleton.builder.insert_point[node_type].append(res)
    return res

class SysBuilder(object):

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.arrays = []
        self.insert_point = {}

    def __enter__(self):
        assert Singleton.builder is None
        Singleton.builder = self
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        assert Singleton.builder is self
        Singleton.builder = None

