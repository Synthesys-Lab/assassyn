from .builder import SysBuilder
from .module import Module, Port
from .expr import Expr
from .block import Block

class Visitor(object):

    def visit_system(self, node: SysBuilder):
        for elem in node.arrays:
            self.visit_array(elem)
        for elem in node.modules:
            self.visit_module(elem)

    def visit_array(self, node):
        pass

    def visit_expr(self, node: Expr):
        pass

    def visit_port(self, node: Port):
        pass

    def visit_module(self, node: Module):
        self.visit_body(node.body)

    def visit_block(self, node: Block):
        for elem in node.body:
            self.dispatch(elem)

    def dispatch(self, node):
        if isinstance(node, Expr):
            self.visit_expr(node)
        if isinstance(node, Block):
            self.visit_block(node)

