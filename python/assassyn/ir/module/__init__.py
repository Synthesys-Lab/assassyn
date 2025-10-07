'''The module for defining the AST nodes for the module and ports.'''

from .module import Module, Port, Wire, combinational
from .downstream import Downstream
from ..memory.dram import DRAM

# For backward compatibility, downstream_combinational is the same as combinational
downstream_combinational = combinational
