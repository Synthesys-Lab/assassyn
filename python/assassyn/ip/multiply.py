from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import assassyn
import pytest
import random

# AIM: an unsigned 32 bit multiplier encapsulation

# MulStage 1: multiply each bit of b
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


# MulStage 2: left shift to multiply weight
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


# Stage 3: add with the final result
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

def multiply(a, b, cnt):
    stage1_reg = RegArray(Int(64), 1)
    stage2_reg = RegArray(Int(64), 1)
    stage3_reg = RegArray(Int(64), 1)

    cycle = RegArray(Int(32),1)
    cycle[0]=cnt-Int(32)(1)
    cond = (cycle[0] < Int(32)(35))&(cnt>=Int(32)(2))

    mulstage1 = MulStage1()
    mulstage1.build(stage1_reg)
    mulstage2 = MulStage2()
    mulstage2.build(stage1_reg, stage2_reg)
    mulstage3 = MulStage3()
    mulstage3.build(stage2_reg, stage3_reg)

    with Condition(cond):
        mulstage1.async_called(a=a, b=b, cnt=cycle[0])
        mulstage2.async_called(cnt=cycle[0])
        mulstage3.async_called(cnt=cycle[0], a=a, b=b)

    return stage3_reg[0]
