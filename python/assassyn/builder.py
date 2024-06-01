
_sys_builder = None

class SysBuilder(object):

    def __init__(self, name):
        self.name = name

    def __enter__(self):
        global _sys_builder
        assert _sys_builder is None
        _sys_builder = self
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        _sys_builder = None
        pass

