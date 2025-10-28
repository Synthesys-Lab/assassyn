'''Base memory module with common functionality for SRAM and DRAM.'''

import math
from ..module.downstream import Downstream
from ..array import RegArray, Array
from ..dtype import Bits
from ..value import Value


class MemoryBase(Downstream):
    '''Base class for memory modules.'''

    # Builtin property of a memory
    width: int      # Width of the memory in bits
    depth: int      # Depth of the memory in words

    # For simulation purpose only
    init_file: str | None  # Path to initialization file
    
    # All the combinational pins into this downstream module.
    we: Value       # Write enable signal
    re: Value       # Read enable signal
    addr: Value     # Address signal
    wdata: Value    # Write data signal
    
    # Derived Values
    addr_width: int # Width of the address in bits
    
    # The array payload as per the depth and width
    _payload: Array  # Array holding the memory contents
    
    def __init__(self, width: int, depth: int, init_file: str | None):
        """Initialize memory base class.
        
        Args:
            width: Width of memory in bits
            depth: Depth of memory in words (must be power of 2)
            init_file: Path to initialization file (can be None)
        """
        super().__init__()
        
        # Validate inputs
        assert isinstance(width, int) and width > 0, f"Width must be positive integer, got {width}"
        assert isinstance(depth, int) and depth > 0, f"Depth must be positive integer, got {depth}"
        assert init_file is None or isinstance(init_file, str), f"Init file must be string or None, got {type(init_file)}"
        
        # Depth is required to be a power of 2
        assert (depth & (depth - 1)) == 0, f"Depth must be a power of 2, got {depth}"
        
        self.width = width
        self.depth = depth
        self.init_file = init_file
        
        # Derive addr_width as log2 of depth
        self.addr_width = int(math.log2(depth))
        
        # Create the payload array with instance-prefixed name
        self._payload = RegArray(
            Bits(width),
            depth,
            attr=[self],
            name=f'{self.name}_val',
            owner=self,
        )
        
        # Initialize signal attributes to None
        self.we = None
        self.re = None
        self.addr = None
        self.wdata = None
