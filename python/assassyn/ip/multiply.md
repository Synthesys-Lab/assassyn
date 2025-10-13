# Unsigned Multiplier IP Module

This module implements a pipelined unsigned 32-bit multiplier that computes the product of two 32-bit integers, producing a 64-bit result. The implementation uses a three-stage pipeline architecture that processes multiplication bit-by-bit using shift-and-add operations.

## Section 0. Summary

The unsigned multiplier implements a classic shift-and-add multiplication algorithm using three pipeline stages. Each stage processes one bit of the multiplier operand, computing the partial product and accumulating it into the final result. The design demonstrates the use of Assassyn's credit-based pipeline architecture for implementing arithmetic operations with proper timing and resource management.

## Section 1. Exposed Interfaces

### multiply

```python
def multiply(a, b, cnt):
    """Compute the product of two 32-bit unsigned integers using a pipelined multiplier.

    Args:
        a: First 32-bit unsigned integer operand
        b: Second 32-bit unsigned integer operand  
        cnt: Counter value indicating the current multiplication step

    Returns:
        The accumulated 64-bit result stored in stage3_reg[0]
    """
```

**Explanation:**

This function orchestrates the three-stage multiplication pipeline. It creates the necessary register arrays for inter-stage communication and instantiates the three multiplication stages. The function uses a cycle counter to control the multiplication process, ensuring that all 32 bits of the multiplier are processed sequentially.

The multiplication algorithm works by:
1. Processing each bit of operand `b` sequentially from bit 0 to bit 31
2. For each bit, computing the partial product and applying the appropriate bit weight
3. Accumulating all partial products to form the final 64-bit result

The function demonstrates the use of Assassyn's asynchronous module calling mechanism, where each stage is triggered based on the cycle counter condition.

## Section 2. Internal Helpers

### MulStage1

```python
class MulStage1(Module):
    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
                'cnt': Port(Int(32)),
            }
        )

    @module.combinational
    def build(self, stage1_reg: Array):
        a, b, cnt = self.pop_all_ports(True)

        with Condition(cnt < Int(32)(32)):# avoid overflow
            b_bit = ((b >> cnt) & Int(32)(1)).bitcast(Int(32))  # to get the cnt-th bit from the right
            stage1_reg[0] = a * b_bit  # 'a' multiply b[cnt-1]
            log("MulStage1: {:?} * {:?} = {:?}", a, b_bit, a * b_bit)
```

**Explanation:**

The first stage extracts the current bit from operand `b` and multiplies it with operand `a`. This implements the bit-wise multiplication step of the shift-and-add algorithm. The stage uses bit manipulation operations to extract the `cnt`-th bit from the right of operand `b`, then performs a simple multiplication to compute the partial product.

The condition `cnt < Int(32)(32)` ensures that only valid bit positions (0-31) are processed, preventing overflow conditions.

### MulStage2

```python
class MulStage2(Module):
    def __init__(self):
        super().__init__(
            ports={
                'cnt': Port(Int(32))
            }
        )

    @module.combinational
    def build(self, stage1_reg: Array, stage2_reg: Array):
        cnt = self.pop_all_ports(True)

        with Condition(cnt > Int(32)(0)):
            bit_num = cnt - Int(32)(1)   # avoid overflow
            with Condition(bit_num < Int(32)(32)):
                stage2_reg[0] = stage1_reg[0] << bit_num  # left shift as multiplying weights
                log("MulStage2: {:?}", stage2_reg[0])
```

**Explanation:**

The second stage applies the appropriate bit weight to the partial product by performing a left shift operation. This implements the "shift" part of the shift-and-add multiplication algorithm. The shift amount is determined by the bit position, where bit `i` has weight `2^i`.

The stage reads the partial product from `stage1_reg` and shifts it left by `bit_num` positions, effectively multiplying by `2^bit_num`. The condition checks ensure that only valid bit positions are processed and that the shift amount doesn't exceed the bit width.

### MulStage3

```python
class MulStage3(Module):
    def __init__(self):
        super().__init__(ports={
            'cnt': Port(Int(32)),
            'a': Port(Int(32)),
            'b': Port(Int(32))
        }
        )

    @module.combinational
    def build(self, stage2_reg: Array, stage3_reg: Array):
        cnt, a, b = self.pop_all_ports(True)
        stage3_reg[0] = stage2_reg[0] + stage3_reg[0]
        log("Temp result {:?} of {:?} * {:?} = {:?}", cnt, a, b, stage3_reg[0])

        with Condition(cnt == Int(32)(34)):  # output final result
            log("Final result {:?} * {:?} = {:?}", a, b, stage3_reg[0])
```

**Explanation:**

The third stage accumulates the weighted partial products into the final result. This implements the "add" part of the shift-and-add multiplication algorithm. The stage adds the current weighted partial product from `stage2_reg` to the accumulated result in `stage3_reg`.

The accumulation continues for all 32 bits of the multiplier, with the final result being available when `cnt == Int(32)(34)`. The extra cycles (32-34) account for the pipeline latency and ensure that all partial products have been processed and accumulated.

The stage demonstrates the use of Assassyn's logging mechanism for debugging and verification purposes, providing visibility into the intermediate and final computation results.
