"""Carry-save unsigned multiplier IP.

The public ``multiply`` helper builds a three-stage 32 by 32 unsigned
multiplier.  The implementation expands the multiplier bits into shifted
partial products, compresses the rows through a 3:2 carry-save tree, and
finishes with one registered carry-propagate add.
"""

from assassyn.frontend import (
    Array,
    Bits,
    Condition,
    Module,
    Port,
    RegArray,
    UInt,
    Value,
    log,
    module,
)


PRODUCT_WIDTH = 64
OPERAND_WIDTH = 32
PARTIAL_PRODUCT_COUNT = 32


def _as_uint(value: Value, width: int) -> Value:
    """Return ``value`` viewed as an unsigned integer of ``width`` bits."""
    dtype = UInt(width)
    if value.dtype == dtype:
        return value
    return value.bitcast(dtype)


def shifted_partial_product(a: Value, b: Value, bit_index: int) -> Value:
    """Build the shifted partial product for one multiplier bit."""
    a64 = _as_uint(a, OPERAND_WIDTH).zext(UInt(PRODUCT_WIDTH))
    b_bits = _as_uint(b, OPERAND_WIDTH).bitcast(Bits(OPERAND_WIDTH))
    selected_bit = (
        (b_bits >> Bits(OPERAND_WIDTH)(bit_index)) & Bits(OPERAND_WIDTH)(1)
    ) != Bits(OPERAND_WIDTH)(0)
    shifted = (a64 << UInt(PRODUCT_WIDTH)(bit_index)).bitcast(UInt(PRODUCT_WIDTH))
    return selected_bit.select(shifted, UInt(PRODUCT_WIDTH)(0))


def carry_save_add(lhs: Value, rhs: Value, third: Value) -> tuple[Value, Value]:
    """Compress three product-width rows into sum and shifted-carry rows."""
    lhs_bits = _as_uint(lhs, PRODUCT_WIDTH).bitcast(Bits(PRODUCT_WIDTH))
    rhs_bits = _as_uint(rhs, PRODUCT_WIDTH).bitcast(Bits(PRODUCT_WIDTH))
    third_bits = _as_uint(third, PRODUCT_WIDTH).bitcast(Bits(PRODUCT_WIDTH))

    sum_bits = (lhs_bits ^ rhs_bits) ^ third_bits
    carry_bits = (
        (lhs_bits & rhs_bits) | (lhs_bits & third_bits) | (rhs_bits & third_bits)
    ) << Bits(PRODUCT_WIDTH)(1)
    return (
        sum_bits.bitcast(UInt(PRODUCT_WIDTH)),
        carry_bits.bitcast(UInt(PRODUCT_WIDTH)),
    )


def carry_save_reduce(rows: list[Value]) -> tuple[Value, Value]:
    """Reduce product rows to the two rows needed by the final adder."""
    current_rows = [_as_uint(row, PRODUCT_WIDTH) for row in rows]
    if not current_rows:
        return UInt(PRODUCT_WIDTH)(0), UInt(PRODUCT_WIDTH)(0)

    while len(current_rows) > 2:
        next_rows = []
        index = 0
        while index + 2 < len(current_rows):
            sum_row, carry_row = carry_save_add(
                current_rows[index],
                current_rows[index + 1],
                current_rows[index + 2],
            )
            next_rows.append(sum_row)
            next_rows.append(carry_row)
            index += 3

        while index < len(current_rows):
            next_rows.append(current_rows[index])
            index += 1

        current_rows = next_rows

    if len(current_rows) == 1:
        current_rows.append(UInt(PRODUCT_WIDTH)(0))
    return current_rows[0], current_rows[1]


class PartialProductStage(Module):
    """Stage 1: register all unsigned shifted partial products."""

    def __init__(self, debug: bool = False):
        super().__init__(
            ports={
                "a": Port(UInt(OPERAND_WIDTH)),
                "b": Port(UInt(OPERAND_WIDTH)),
                "tag": Port(UInt(OPERAND_WIDTH)),
            }
        )
        self.debug = debug

    @module.combinational
    def build(
        self,
        partial_products: list[Array],
        stage1_a: Array,
        stage1_b: Array,
        stage1_tag: Array,
    ):
        a, b, tag = self.pop_all_ports(True)

        for bit_index in range(PARTIAL_PRODUCT_COUNT):
            partial_products[bit_index][0] = shifted_partial_product(a, b, bit_index)

        stage1_a[0] = a
        stage1_b[0] = b
        stage1_tag[0] = tag

        if self.debug:
            log("CsaMultiplierStage1: tag={} a={} b={}", tag, a, b)


class CarrySaveStage(Module):
    """Stage 2: reduce the registered partial products with 3:2 compressors."""

    def __init__(self, debug: bool = False):
        super().__init__(ports={"valid": Port(UInt(1))})
        self.debug = debug

    @module.combinational
    def build(
        self,
        partial_products: list[Array],
        stage1_a: Array,
        stage1_b: Array,
        stage1_tag: Array,
        stage2_sum: Array,
        stage2_carry: Array,
        stage2_a: Array,
        stage2_b: Array,
        stage2_tag: Array,
    ):
        valid = self.pop_all_ports(True)

        with Condition(valid != UInt(1)(0)):
            rows = [partial_products[index][0] for index in range(PARTIAL_PRODUCT_COUNT)]
            sum_row, carry_row = carry_save_reduce(rows)
            stage2_sum[0] = sum_row
            stage2_carry[0] = carry_row
            stage2_a[0] = stage1_a[0]
            stage2_b[0] = stage1_b[0]
            stage2_tag[0] = stage1_tag[0]

            if self.debug:
                log(
                    "CsaMultiplierStage2: tag={} sum={} carry={}",
                    stage1_tag[0],
                    sum_row,
                    carry_row,
                )


class FinalAddStage(Module):
    """Stage 3: register the carry-propagate product."""

    def __init__(self, debug: bool = False):
        super().__init__(ports={"valid": Port(UInt(1))})
        self.debug = debug

    @module.combinational
    def build(
        self,
        stage2_sum: Array,
        stage2_carry: Array,
        stage2_a: Array,
        stage2_b: Array,
        stage2_tag: Array,
        result: Array,
    ):
        valid = self.pop_all_ports(True)

        with Condition(valid != UInt(1)(0)):
            product = stage2_sum[0] + stage2_carry[0]
            result[0] = product

            if self.debug:
                log(
                    "CsaMultiplierResult: tag={} a={} b={} product={}",
                    stage2_tag[0],
                    stage2_a[0],
                    stage2_b[0],
                    product,
                )


def multiply(a: Value, b: Value, cnt: Value, debug: bool = False) -> Value:
    """Build and call the shared three-stage unsigned multiplier pipeline."""
    partial_products = [
        RegArray(UInt(PRODUCT_WIDTH), 1) for _ in range(PARTIAL_PRODUCT_COUNT)
    ]
    stage1_a = RegArray(UInt(OPERAND_WIDTH), 1)
    stage1_b = RegArray(UInt(OPERAND_WIDTH), 1)
    stage1_tag = RegArray(UInt(OPERAND_WIDTH), 1)
    stage2_sum = RegArray(UInt(PRODUCT_WIDTH), 1)
    stage2_carry = RegArray(UInt(PRODUCT_WIDTH), 1)
    stage2_a = RegArray(UInt(OPERAND_WIDTH), 1)
    stage2_b = RegArray(UInt(OPERAND_WIDTH), 1)
    stage2_tag = RegArray(UInt(OPERAND_WIDTH), 1)
    result = RegArray(UInt(PRODUCT_WIDTH), 1)

    partial_stage = PartialProductStage(debug=debug)
    partial_stage.build(partial_products, stage1_a, stage1_b, stage1_tag)
    carry_save_stage = CarrySaveStage(debug=debug)
    carry_save_stage.build(
        partial_products,
        stage1_a,
        stage1_b,
        stage1_tag,
        stage2_sum,
        stage2_carry,
        stage2_a,
        stage2_b,
        stage2_tag,
    )
    final_stage = FinalAddStage(debug=debug)
    final_stage.build(stage2_sum, stage2_carry, stage2_a, stage2_b, stage2_tag, result)

    partial_stage.async_called(
        a=_as_uint(a, OPERAND_WIDTH),
        b=_as_uint(b, OPERAND_WIDTH),
        tag=_as_uint(cnt, OPERAND_WIDTH),
    )
    carry_save_stage.async_called(valid=UInt(1)(1))
    final_stage.async_called(valid=UInt(1)(1))

    return result[0]
