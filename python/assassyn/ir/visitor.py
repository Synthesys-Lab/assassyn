'''The module for the frontend AST visitor pattern'''

from ..builder import SysBuilder
from .module import Module, Port
from .expr import Expr

class Visitor:
    '''The visitor pattern class for the frontend AST'''
    # Base visitor class with no attributes of its own - it just defines visit methods

    current_module: Module

    def __init__(self):
        '''Initialize the visitor with no current module'''
        self.current_module = None

    def visit_system(self, node: SysBuilder):
        '''Enter a system'''
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            self.current_module = elem
            self.visit_module(elem)
        self.current_module = None
        for elem in node.downstreams:
            self.visit_module(elem)

    def visit_array(self, node):
        '''Enter an array'''

    def visit_expr(self, node: Expr):
        '''Enter an expression'''

    def visit_port(self, node: Port):
        '''Enter a port'''

    def visit_module(self, node: Module):
        '''Enter a module'''
        body = getattr(node, "body", None)
        if isinstance(body, list):
            for elem in body:
                self.dispatch(elem)

    def dispatch(self, node):
        '''Dispatch the node in a block to the corresponding visitor'''
        if isinstance(node, Expr):
            self.visit_expr(node)
