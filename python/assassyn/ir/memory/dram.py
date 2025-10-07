'''DRAM memory module implementation.'''

from .base import MemoryBase
from ..module.downstream import combinational
from ..block import Condition
from ..expr.intrinsic import (
    send_read_request, send_write_request, 
    read_request_succ, write_request_succ
)


class DRAM(MemoryBase):
    '''The DRAM module, a subclass of MemoryBase.
    
    This module simulates an off-chip DRAM module that interacts with 
    the on-chip pipeline. Unlike SRAM, the data should be handled as 
    soon as response, using several intrinsics to achieve this.
    '''

    def __init__(self, width: int, depth: int, init_file: str | None):
        """Initialize DRAM module.
        
        Args:
            width: Width of memory in bits
            depth: Depth of memory in words (must be power of 2)
            init_file: Path to initialization file (can be None)
        """
        super().__init__(width, depth, init_file)

    @combinational
    def build(self, we, re, addr, wdata):  # pylint: disable=too-many-arguments
        '''The constructor for the DRAM module.

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

        # When re is enabled, call send_read_request
        with Condition(re):
            send_read_request(self, addr)
            
        # When we is enabled, call send_write_request
        with Condition(we):
            send_write_request(self, addr, wdata)
            
        # Return success signals for downstream modules to check
        # It is developers' duty to resend unsuccessful requests
        return read_request_succ(self), write_request_succ(self)

    def __repr__(self):
        return self._repr_impl('memory.DRAM')