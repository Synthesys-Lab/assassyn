"""Utility functions for the Verilog backend."""

from collections import deque
from typing import Iterator, List, Optional, Tuple

from ...ir.array import Array
from ...ir.module import Port, Module
from ...ir.expr import Intrinsic
from ...ir.dtype import Int, UInt, Bits, DType
from ...utils import namify, identifierize

class DisplayInstance:
    """A display instance for a module element."""

    def __init__(self, prefix: str, id_: str):
        self.prefix = prefix
        self.id = id_

    @classmethod
    def from_module(cls, module: Module) -> 'DisplayInstance':
        """Create a display instance from a module."""
        return cls("", identifierize(module.name))

    @classmethod
    def from_array(cls, array: Array) -> 'DisplayInstance':
        """Create a display instance from an array."""
        return cls("array", namify(array.name))

    @classmethod
    def from_fifo(cls, fifo: Port, global_: bool) -> 'DisplayInstance':
        """Create a display instance from a FIFO."""
        raw = namify(fifo.name)
        fifo_name = f"{namify(fifo.module.name)}_{raw}" if global_ else raw
        return cls("fifo", fifo_name)

    def field(self, attr: str) -> str:
        """Get the field of the display instance."""
        return f"{self}_{attr}"

    def __str__(self) -> str:
        if not self.prefix:
            return self.id
        return f"{self.prefix}_{self.id}"

class Edge:
    """An edge in the module graph."""

    def __init__(self, instance: DisplayInstance, driver: Module):
        self.instance = instance
        self.driver = identifierize(driver.name)

    def field(self, field: str) -> str:
        """Get the field of the edge."""
        return f"{self.instance}_driver_{self.driver}_{field}"

def broadcast(value: str, bits: int) -> str:
    """Broadcast a value to a specific bit width."""
    return f"{{ {bits} {{ {value} }} }}"

def select_1h(iter_: Iterator[Tuple[str, str]], bits: int) -> str:
    """Select a value from a one-hot encoding."""
    return reduce(
        map(lambda x: f"({broadcast(x[0], bits)} & {x[1]})", iter_),
        " | "
    )

def reduce(iter_: Iterator[str], concat: str) -> str:
    """Reduce a sequence of strings to a single string."""
    result = list(iter_)
    if not result:
        return "'x"
    return concat.join(result)

def bool_ty() -> DType:
    """Create a boolean type."""
    return Int(1)

def declare_impl(decl_prefix: str, ty: DType, id_: str, term: str) -> str:
    """Declare a variable in Verilog."""
    bits = ty.bits - 1
    return f"  {decl_prefix} [{bits}:0] {id_}{term}\n"

def declare_logic(ty: DType, id_: str) -> str:
    """Declare a logic variable in Verilog."""
    return declare_impl("logic", ty, id_, ";")

def declare_in(ty: DType, id_: str) -> str:
    """Declare an input in Verilog."""
    return declare_impl("input logic", ty, id_, ",")

def declare_out(ty: DType, id_: str) -> str:
    """Declare an output in Verilog."""
    return declare_impl("output logic", ty, id_, ",")

def declare_array(prefix: str, array: Array, id_: str, term: str) -> str:
    """Declare an array in Verilog."""
    size = array.size
    ty = array.scalar_ty
    prefix_str = f"{prefix} " if prefix else ""
    return f"  {prefix_str}logic [{(ty.bits * size) - 1}:0] {id_}{term}\n"

def connect_top(display, edge, fields: List[str]) -> str:
    """Connect a display instance to an edge."""
    result = ""
    for field in fields:
        result += f"    .{display.field(field)}({edge.field(field)}),\n"
    return result

def type_to_fmt(ty: DType) -> str:
    """Convert a type to a format string."""
    if isinstance(ty, (Int, UInt, Bits)):
        return "d"
    raise ValueError(f"Invalid type for type: {ty}")

def parse_format_string(args: List) -> str:
    """Parse a format string for log statements."""
    assert isinstance(args[0], str)
    raw = args[0]
    fmt = deque(raw)
    result = ""
    arg_idx = 1

    while fmt:
        c = fmt.popleft()
        if c == '{':
            if fmt and fmt[0] == '{':
                fmt.popleft()
                result += '{'
            else:
                dtype = args[arg_idx].dtype
                substr = ""
                if not fmt:
                    raise ValueError("Invalid format string, missing closing brace")

                next_char = fmt.popleft()
                if next_char == '}':
                    result += f"%{type_to_fmt(dtype)}"
                    arg_idx += 1
                    continue

                substr = next_char
                closed = False
                while fmt:
                    next_char = fmt.popleft()
                    if next_char == '}':
                        closed = True
                        break
                    substr += next_char

                if not closed:
                    raise ValueError(f"Invalid format string, missing closing brace: {raw}")

                if not substr:
                    new_fmt = type_to_fmt(dtype)
                elif substr.startswith(':'):
                    width_idx = 1
                    pad = "0" if substr[1:2] == "0" else ""
                    width_idx += 1 if pad else 0

                    width = ""
                    if width_idx < len(substr) and substr[width_idx].isdigit():
                        width = substr[width_idx]
                        width_idx += 1

                    vfmt = type_to_fmt(dtype)
                    if width_idx < len(substr) and substr[width_idx].isalpha():
                        vfmt = substr[width_idx]

                    new_fmt = f"%{pad}{width}{vfmt}"
                else:
                    raise ValueError(f"Invalid format string: {raw}")

                result += new_fmt
                arg_idx += 1
        elif c == '}':
            if fmt and fmt[0] == '}':
                fmt.popleft()
                result += '}'
            else:
                raise ValueError("Invalid format string, because of a single }")
        else:
            result += c

    return result

def find_wait_until(module: Module) -> Optional[Intrinsic]:
    """Find the WAIT_UNTIL intrinsic in a module if it exists."""
    for elem in module.body.body:
        if isinstance(elem, Intrinsic):
            if elem.opcode == Intrinsic.WAIT_UNTIL:
                return elem
    return None
