'''The module for intrinsic expressions'''
#pylint: disable=cyclic-import

from ...builder import ir_builder
from .expr import Expr

INTRIN_INFO = {
    # Intrinsic operations opcode: (mnemonic, num of args, valued, side effect)
    900: ('wait_until', 1, False, True),
    901: ('finish', 0, False, True),
    902: ('assert', 1, False, True),
    914: ('PUSH_CONDITION', 1, False, True),
    915: ('POP_CONDITION', 0, False, True),
    906: ('send_read_request', 3, True, True),
    908: ('send_write_request', 4, True, True),
    913: ('external_instantiate', None, True, True),  # None = variable args
}

PURE_INTRIN_INFO = {
    # PureIntrinsic operations opcode: (mnemonic, num of args)
    307: ('current_cycle', 0),
    306: ('external_output_read', None),  # (instance, port_name[, index]) - variable args
    904: ('has_mem_resp', 1),
    912: ('get_mem_resp', 1),
}

class Intrinsic(Expr):
    '''The class for intrinsic operations'''

    WAIT_UNTIL = 900
    FINISH = 901
    ASSERT = 902
    SEND_READ_REQUEST = 906
    SEND_WRITE_REQUEST = 908
    EXTERNAL_INSTANTIATE = 913
    PUSH_CONDITION = 914
    POP_CONDITION = 915

    opcode: int  # Operation code for this intrinsic

    def __init__(self, opcode, *args, meta_cond=None):
        payload = list(args)
        _, num_args, _, _ = INTRIN_INFO[opcode]
        # num_args can be None for variable-length args (like EXTERNAL_INSTANTIATE)
        if num_args is not None:
            assert len(payload) == num_args
        super().__init__(opcode, payload, meta_cond=meta_cond)
        self._payload_len = len(payload)

    @property
    def args(self):
        '''Get the arguments of this intrinsic.'''
        return self._operands[:self._payload_len]
    @property
    def dtype(self):
        '''Get the data type of this intrinsic.'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        if self.opcode in [Intrinsic.SEND_READ_REQUEST, Intrinsic.SEND_WRITE_REQUEST]:
            return Bits(1)
        return Bits(1)

    def __repr__(self):
        args = {", ".join(i.as_operand() for i in self.args[0:])}
        mn, _, valued, side_effect = INTRIN_INFO[self.opcode]
        side_effect = ['', 'side effect '][side_effect]
        rhs = f'{side_effect}intrinsic.{mn}({args})'
        if valued:
            return f'{self.as_operand()} = {rhs}'
        return rhs

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

@ir_builder
def wait_until(cond):
    '''Frontend API for creating a wait-until block.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    assert isinstance(cond, Value)
    return Intrinsic(Intrinsic.WAIT_UNTIL, cond)

@ir_builder
def assume(cond):
    '''Frontend API for creating an assertion.
    This name is to avoid conflict with the Python keyword.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    assert isinstance(cond, Value)
    return Intrinsic(Intrinsic.ASSERT, cond)


def is_wait_until(expr):
    '''Check if the expression is a wait-until intrinsic.'''
    return isinstance(expr, Intrinsic) and expr.opcode == Intrinsic.WAIT_UNTIL

@ir_builder
def finish():
    '''Finish the simulation.'''
    return Intrinsic(Intrinsic.FINISH)

@ir_builder
def has_mem_resp(memory):
    '''Check if there is a memory response.'''
    return PureIntrinsic(PureIntrinsic.HAS_MEM_RESP, memory)


@ir_builder
def send_read_request(mem, re, addr):
    '''Send a read request with address to the given memory system.'''
    return Intrinsic(Intrinsic.SEND_READ_REQUEST, mem, re, addr)

@ir_builder
def send_write_request(mem, we, addr, data):
    '''Send a write request with address and data to the given memory system.'''
    return Intrinsic(Intrinsic.SEND_WRITE_REQUEST, mem, we, addr, data)


@ir_builder
def push_condition(cond):
    '''Push a predicate condition to the builder condition stack and IR.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    from ...builder import Singleton
    assert isinstance(cond, Value)
    # Mirror into builder predicate stack for frontend semantics (per module)
    Singleton.peek_builder().push_predicate(cond)
    return Intrinsic(Intrinsic.PUSH_CONDITION, cond)


@ir_builder
def pop_condition():
    '''Pop a predicate condition from the builder condition stack and IR.'''
    #pylint: disable=import-outside-toplevel
    from ...builder import Singleton
    Singleton.peek_builder().pop_predicate()
    return Intrinsic(Intrinsic.POP_CONDITION)



@ir_builder
def get_mem_resp(mem):
    '''Get the memory response data. The lsb are the data payload,
    and the msb are the corresponding request address.'''
    return PureIntrinsic(PureIntrinsic.GET_MEM_RESP, mem)

class PureIntrinsic(Expr):
    '''The class for accessing FIFO fields, valid, and peek'''

    # FIFO operations
    FIFO_VALID = 300
    FIFO_PEEK  = 303
    MODULE_TRIGGERED = 304
    VALUE_VALID = 305
    CURRENT_CYCLE = 307

    # External module operations
    EXTERNAL_OUTPUT_READ = 306  # Unified opcode for both wire and reg outputs
    # Deprecated aliases (for backward compatibility)
    EXTERNAL_WIRE_OUT = 306
    EXTERNAL_REG_OUT = 306

    # Memory response operations
    HAS_MEM_RESP = 904
    GET_MEM_RESP = 912

    OPERATORS = {
        FIFO_VALID: 'valid',
        FIFO_PEEK: 'peek',
        MODULE_TRIGGERED: 'triggered',
        VALUE_VALID: 'valid',
    }

    def __init__(self, opcode, *args, meta_cond=None):
        operands = list(args)
        # Validate arguments for operations with defined arg counts
        if opcode in PURE_INTRIN_INFO:
            _, num_args = PURE_INTRIN_INFO[opcode]
            if num_args is not None:
                assert len(args) == num_args, \
                    f"Expected {num_args} args for opcode {opcode}, got {len(args)}"
        super().__init__(opcode, operands, meta_cond=meta_cond)
        self._payload_len = len(operands)

    @property
    def args(self):
        '''Get the arguments of this intrinsic'''
        return self._operands[:self._payload_len]

    @property
    def dtype(self):
        '''Get the data type of this intrinsic'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits, UInt

        if self.opcode == PureIntrinsic.FIFO_PEEK:
            # pylint: disable=import-outside-toplevel
            from ..module import Port
            fifo = self.args[0]
            assert isinstance(fifo, Port)
            return fifo.dtype

        if self.opcode in [PureIntrinsic.FIFO_VALID, PureIntrinsic.MODULE_TRIGGERED,
                           PureIntrinsic.VALUE_VALID, PureIntrinsic.HAS_MEM_RESP]:
            return Bits(1)

        if self.opcode == PureIntrinsic.GET_MEM_RESP:
            return Bits(self.args[0].width)

        if self.opcode == PureIntrinsic.CURRENT_CYCLE:
            return UInt(64)

        if self.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ:
            # args[0] is ExternalIntrinsic instance, args[1] is port name
            # args[2] (optional) is index for RegOut
            instance = self.args[0]
            port_name = self.args[1].value if hasattr(self.args[1], 'value') else self.args[1]
            return instance.get_output_dtype(port_name)

        raise NotImplementedError(f'Unsupported intrinsic operation {self.opcode}')

    def __repr__(self):
        if self.opcode in [PureIntrinsic.FIFO_PEEK, PureIntrinsic.FIFO_VALID,
                           PureIntrinsic.MODULE_TRIGGERED, PureIntrinsic.VALUE_VALID]:
            fifo = self.args[0].as_operand()
            return f'{self.as_operand()} = {fifo}.{self.OPERATORS[self.opcode]}()'
        if self.opcode in [PureIntrinsic.HAS_MEM_RESP, PureIntrinsic.GET_MEM_RESP,
                           PureIntrinsic.CURRENT_CYCLE]:
            mn, _ = PURE_INTRIN_INFO[self.opcode]
            args = ", ".join(i.as_operand() for i in self.args)
            return f'{self.as_operand()} = pure_intrinsic.{mn}({args})'
        if self.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ:
            inst = self.args[0].as_operand()
            port = self.args[1].value if hasattr(self.args[1], 'value') else self.args[1]
            if len(self.args) > 2:  # Has index (RegOut)
                idx = self.args[2].as_operand()
                return f'{self.as_operand()} = {inst}.{port}[{idx}]'
            return f'{self.as_operand()} = {inst}.{port}'
        raise NotImplementedError

    def __getattr__(self, name):
        if self.opcode == PureIntrinsic.FIFO_PEEK:
            port = self.args[0]
            # pylint: disable=import-outside-toplevel
            from ..module import Port
            assert isinstance(port, Port)
            return port.dtype.attributize(self, name)

        assert False, f"Cannot access attribute {name} on {self}"


@ir_builder
def current_cycle():
    '''Frontend API to get current global cycle (UInt(64)).'''
    return PureIntrinsic(PureIntrinsic.CURRENT_CYCLE)


## CURRENT_CYCLE alias removed; use current_cycle() instead.


# Frontend helper: get_pred (no opcode)
def get_pred():
    '''Get the current predicate as AND of builder condition stack.'''
    #pylint: disable=import-outside-toplevel
    from ...builder import Singleton
    return Singleton.peek_builder().current_predicate_carry()


class ExternalIntrinsic(Intrinsic):
    """Intrinsic representing external module instantiation.

    This intrinsic creates an instance of an external SystemVerilog module
    and connects its input ports. Output ports are accessed via attribute
    access, which creates PureIntrinsic read operations.

    Args:
        external_class: The ExternalSV subclass defining the module
        **input_connections: Dict of port_name -> Value mappings for inputs
    """

    def __init__(self, external_class, **input_connections):
        # pylint: disable=import-outside-toplevel
        from ..module.external import ExternalSV
        assert issubclass(external_class, ExternalSV), \
            f"{external_class} must be a subclass of ExternalSV"

        self._external_class = external_class
        self._input_connections = input_connections

        port_specs = external_class.port_specs()

        # Validate all required inputs are provided
        for name, wire_spec in port_specs.items():
            if wire_spec.direction == 'in':
                assert name in input_connections, \
                    f"Missing required input '{name}' for {external_class.__name__}"

        # Validate no extra inputs
        for name in input_connections:
            assert name in port_specs, \
                f"Unknown port '{name}' for {external_class.__name__}"
            wire_spec = port_specs[name]
            assert wire_spec.direction == 'in', \
                f"Port '{name}' is not an input port"

        # Store input values as operands for IR traversal
        operands = list(input_connections.values())
        super().__init__(Intrinsic.EXTERNAL_INSTANTIATE, *operands)

    @property
    def external_class(self):
        """Get the ExternalSV class this intrinsic instantiates."""
        return self._external_class

    @property
    def input_connections(self):
        """Get the dict of input port connections."""
        return self._input_connections

    @property
    def uid(self):
        """Get unique identifier for this intrinsic instance."""
        return id(self)

    def get_output_dtype(self, port_name):
        """Get the dtype of an output port.

        Args:
            port_name: Name of the output port

        Returns:
            The dtype of the specified output port
        """
        port_specs = self._external_class.port_specs()
        assert port_name in port_specs, \
            f"Unknown port '{port_name}'"
        wire_spec = port_specs[port_name]
        assert wire_spec.direction == 'out', \
            f"{port_name} is not an output port"
        return wire_spec.dtype

    def __getattr__(self, name):
        """Access output ports by attribute.

        For WireOut ports: returns a PureIntrinsic that reads the value
        For RegOut ports: returns an array proxy that supports indexing

        Args:
            name: Name of the output port to access

        Returns:
            PureIntrinsic for WireOut, or _ExternalRegOutProxy for RegOut
        """
        # Don't intercept private/internal attributes
        if name.startswith('_'):
            raise AttributeError(
                f"'{type(self).__name__}' object has no attribute '{name}'")

        # Check if it's a valid output port
        port_specs = self._external_class.port_specs()
        wire_spec = port_specs.get(name)
        if wire_spec is None:
            raise AttributeError(
                f"Unknown port '{name}' in {self._external_class.__name__}")

        if wire_spec.direction != 'out':
            raise AttributeError(
                f"Cannot read input port '{name}'")

        # Return different representation based on port kind
        if wire_spec.kind == 'wire':
            # WireOut: return PureIntrinsic directly (no index)
            @ir_builder
            def _read():
                return PureIntrinsic(PureIntrinsic.EXTERNAL_OUTPUT_READ, self, name)
            return _read()

        if wire_spec.kind == 'reg':
            # RegOut: return array proxy (will add index when accessed)
            # pylint: disable=import-outside-toplevel
            from ..module.external import _ExternalRegOutProxy
            return _ExternalRegOutProxy(self, name, wire_spec.dtype)

        raise NotImplementedError(f"Unknown wire kind {wire_spec.kind}")

    @property
    def dtype(self):
        """ExternalIntrinsic returns Bits(1) indicating instantiation success."""
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        return Bits(1)

    def __repr__(self):
        """String representation for debugging."""
        inputs = ", ".join(f"{k}={v.as_operand()}"
                          for k, v in self._input_connections.items())
        return (f'{self.as_operand()} = external_instantiate.'
                f'{self._external_class.__name__}({inputs})')
