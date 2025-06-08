"""Utility functions for simulator generation."""

from ...ir.dtype import DType, Void, ArrayType, Record, Bits
from ...ir.module import Port
from ...utils import namify

def camelize(name: str) -> str:
    """Convert a name to camelCase.

    This matches the Rust function in src/backend/simulator/utils.rs
    """
    result = ""
    capitalize = True
    for c in name:
        if c == '_':
            capitalize = True
        elif capitalize:
            result += c.upper()
            capitalize = False
        else:
            result += c
    return result


def dtype_to_rust_type(dtype: DType) -> str:
    # disable=too-many-return-statements
    """Convert an Assassyn data type to a Rust type.

    This matches the Rust function in src/backend/simulator/utils.rs
    """

    if isinstance(dtype, Record):
        dtype = Bits(dtype.bits)

    if dtype.is_int() or dtype.is_raw():
        prefix = "u" if not dtype.is_signed() or dtype.is_raw() else "i"
        bits = dtype.bits

        if 8 <= bits <= 64:
            # Round up to next power of 2
            bits = 1 << (bits - 1).bit_length()
            return f"{prefix}{bits}"
        if bits == 1:
            return "bool"
        if bits < 8:
            return f"{prefix}8"
        if bits > 64:
            return 'BigUint' if not dtype.is_signed() or dtype.is_raw() else 'BigInt'
        raise ValueError(f"Unsupported data type: {dtype}")

    if isinstance(dtype, Void):
        return "Box<EventKind>"

    if isinstance(dtype, ArrayType):
        elem_ty = dtype_to_rust_type(dtype.scalar_ty)
        size = dtype.size
        return f"[{elem_ty}; {size}]"
    raise ValueError(f"Unsupported data type: {dtype}")


def int_imm_dumper_impl(ty: DType, value: int) -> str:
    """Generate Rust code for integer immediate values.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
    if ty.bits == 1:
        return "true" if value != 0 else "false"

    if ty.bits <= 64:
        return f"{value}{dtype_to_rust_type(ty)}"

    scalar_ty = "i64" if ty.is_signed() else "u64"
    return f"ValueCastTo::<{dtype_to_rust_type(ty)}>::cast(&({value} as {scalar_ty}))"


def fifo_name(fifo: Port):
    """Generate a name for a FIFO.

    This matches the Rust macro in src/backend/simulator/elaborate.rs
    """
    module = fifo.module
    return f"{namify(module.name)}_{namify(fifo.name)}"
