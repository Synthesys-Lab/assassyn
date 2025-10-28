'''SRAM memory module implementation.'''

from .base import MemoryBase
from ..module.downstream import combinational
from ..block import Condition
from ..array import RegArray
from ..dtype import Bits
from ..expr import assume


class SRAM(MemoryBase):  # pylint: disable=too-many-instance-attributes
    '''The SRAM module, a subclass of MemoryBase.'''
    
    # Additional attributes specific to SRAM
    dout: RegArray  # Register buffer that holds the result of read

    def __init__(self, width: int, depth: int, init_file: str | None):
        """Initialize SRAM module.
        
        Args:
            width: Width of memory in bits
            depth: Depth of memory in words (must be power of 2)
            init_file: Path to initialization file (can be None)
        """
        super().__init__(width, depth, init_file)
        # Create dout register buffer with instance-prefixed name
        self.dout = RegArray(
            Bits(width),
            1,
            name=f'{self.name}_rdata',
            owner=self,
        )

    @combinational
    def build(self, we, re, addr, wdata):  # pylint: disable=too-many-arguments
        '''The constructor for the SRAM module.

        Args:
            we: Value: The write enable signal.
            re: Value: The read enable signal.
            addr: Value: The address signal.
            wdata: Value: The write data signal.
        '''
        self.we = we
        self.re = re
        self.addr = addr
        self.wdata = wdata

        # Enforce that we and re cannot be both enabled
        assume(~(we & re))

        with Condition(we):
            self._payload[addr] = wdata
        with Condition(re):
            self.dout[0] = self._payload[addr]

    def __repr__(self):
        return self._repr_impl('memory.SRAM')
