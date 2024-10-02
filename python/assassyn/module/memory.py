'''Memory module, a special and subclass of Module.'''

from .downstream import Downstream, combinational
from ..array import RegArray
from ..block import Condition
from ..builder import ir_builder
from ..dtype import Bits


class SRAM(Downstream):

    def __init__(self, width, depth, init_file):
        super().__init__()
        self.width = width
        self.depth = depth
        self.init_file = init_file
        self.payload = RegArray(Bits(width), depth, attr=[self])

    @combinational
    def build(self, we, re, addr, wdata, user):
        '''The constructor for the SRAM module.

        # Arguments
        init_file: str: The file to initialize the memory.
        we: Value: The write enable signal.
        re: Value: The read enable signal.
        addr: Value: The address signal.
        wdata: Value: The write data signal.
        user: Module: The user module, it is required to have a rdata port.

        # Returns
        bound: Bind: The bound handle of the user module.
        '''
        self.we = we
        self.re = re
        self.addr = addr
        self.wdata = wdata

        with Condition(we):
            payload[addr] = wdata
        with Condition(re):
            self.bound = user.bind(rdata=payload[addr])

        return self.bound

    def __repr__(self):
        var_id = self.as_operand()
        body = self.body.__repr__() if self.body is not None else ''
        return f'''  #[downstream.SRAM]
  {var_id} = module {self.name} {{
{body}
  }}
'''
