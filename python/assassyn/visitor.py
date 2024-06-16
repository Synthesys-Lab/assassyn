'''The module for the frontend AST visitor pattern'''

from .builder import SysBuilder
from .module import Module, Port
from .expr import Expr
from .block import Block

class Visitor:
    '''The visitor pattern class for the frontend AST'''

    def visit_system(self, node: SysBuilder):
        '''Enter a system'''
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            self.visit_module(elem)

    def visit_array(self, node):
        '''Enter an array'''

    def visit_expr(self, node: Expr):
        '''Enter an expression'''

    def visit_port(self, node: Port):
        '''Enter a port'''

    def visit_module(self, node: Module):
        '''Enter a module'''
        self.visit_block(node.body)

    def visit_block(self, node: Block):
        '''Enter a block'''
        for elem in node.body:
            self.dispatch(elem)

    def dispatch(self, node):
        '''Dispatch the node in a block to the corresponding visitor'''
        if isinstance(node, Expr):
            self.visit_expr(node)
        if isinstance(node, Block):
            self.visit_block(node)
