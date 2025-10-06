'''Memory module, a special and subclass of Module.'''

from .downstream import Downstream
from ..memory.base import MemoryBase
from .downstream import combinational as downstream_combinational
from .module import combinational as module_combinational
from ..array import Array
from ..block import Condition
from ..dtype import Bits
from ..expr import Bind
from ..value import Value
from ..expr import mem_write, send_read_request, has_mem_resp, send_write_request, use_dram

class DRAM(MemoryBase): # pylint: disable=too-many-instance-attributes, duplicate-code
    '''The DRAM module.'''
    #pylint: disable=duplicate-code
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
        super().__init__(width, depth, init_file)
        self.bound = None

    @module_combinational
    def build(self, we, re, addr, wdata, handle_response): #pylint: disable=too-many-arguments
        '''The constructor for the DRAM module.

        # Arguments
        init_file: str: The file to initialize the memory.
        we: Value: The write enable signal.
        re: Value: The read enable signal.
        addr: Value: The address signal.
        wdata: Value: The write data signal.
        handle_response: another module

        # Returns
        bound: Bind: The bound handle of the user module.
        '''
        dram_handler = DramHandler(self.width, handle_response, we, re)
        self.bound = dram_handler.build(self._payload, addr, wdata)
        return self.bound

    def __repr__(self):
        return f'DRAM(width={self.width}, depth={self.depth}, init_file={self.init_file})'

class DramHandler(Downstream):
    """"Dram handler class"""        

    def __init__(self, width, handle_response, we, re):
        super().__init__()
        self.we = we
        self.re = re
        self.width = width
        self.handle_response = handle_response
        self.bound = None

    @downstream_combinational
    def build(self, payload, addr, wdata):
        """Build the DRAM handler configuration.
        
        Returns:
            bound: The bound handle
        """
        # kind_we = Bits(1)(0)
        # kind_re = Bits(1)(0)
        succ = send_write_request(addr, self.we)

        kind_we = self.we
        with Condition(succ):
            mem_write(payload, addr, wdata)
        with Condition(self.re):
            send_read_request(addr)
        has_resp = has_mem_resp(self)
        x = use_dram(self.handle_response.mem)
        x.fifo = self.handle_response.mem
        x.val = self.handle_response.mem
        self.bound = self.handle_response.bind()
        self.bound.pushes.append(x)
        kind_re = has_resp.select(Bits(1)(1), Bits(1)(0))
        with Condition(self.we | has_resp):
            self.bound.bind(kind_we = kind_we,
                            kind_re = kind_re,
                            write_success = succ)
        return self.bound
