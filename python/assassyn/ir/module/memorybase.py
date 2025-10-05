'''Base memory module with common functionality for SRAM and DRAM.'''

from ..array import RegArray, Array
from ..dtype import Bits
from ..expr import Bind
from ..value import Value

class MemoryBase: # pylint: disable=too-many-instance-attributes,too-few-public-methods, duplicate-code
    '''Base class for memory modules.'''

    width: int  # Width of the memory in bits
    depth: int  # Depth of the memory in words
    init_file: str  # Path to initialization file
    payload: Array  # Array holding the memory contents
    we: Value  # Write enable signal
    re: Value  # Read enable signal
    addr: Value  # Address signal
    wdata: Value  # Write data signal
    bound: Bind  # Bind handle

    def __init__(self, width, depth, init_file):
        """Initialize memory base class.
        
        Args:
            width: Width of memory in bits
            depth: Depth of memory in words
            init_file: Path to initialization file
        """
        self.width = width
        self.depth = depth
        self.init_file = init_file
        self.payload = RegArray(Bits(width), depth, attr=[self])
        self.we = None
        self.re = None
        self.addr = None
        self.wdata = None
        self.bound = None
