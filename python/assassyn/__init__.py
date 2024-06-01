"""Assassyn's python frontend."""

from .dtype import *
from .builder import *

class Module(object):
    def __init__(self):
        self.a = Port(DType())
        pass

class Port(object):
    def __init__(self, dtype: DType):
        pass

    def valid(self):
        pass

    def peek(self):
        pass

    def pop(self):
        pass

    def push(self):
        pass

