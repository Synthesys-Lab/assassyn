from assassyn.frontend import *
from assassyn.test import run_test
import random

# AIM: unsigned 32 bit multiplier: 32b*32b=64b
# DATE: 2025/4/16

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
            (stage1_reg&self)[0] <= a * b_bit  # 'a' multiply b[cnt-1]
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
                # left shift to multiply weight
                (stage2_reg & self)[0] <= (stage1_reg[0] << bit_num).bitcast(Int(64))
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
        (stage3_reg&self)[0] <= stage2_reg[0] + stage3_reg[0]
        log("MulStage3: {:?}", stage3_reg[0])
        log("Temp result {:?} of {:?} * {:?} = {:?}", cnt, a, b, stage3_reg[0])

        with Condition(cnt == Int(32)(34)):  # output final result
            log("Final result {:?} * {:?} = {:?}", a, b, stage3_reg[0])


class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, mulstage1: MulStage1, mulstage2: MulStage2, mulstage3: MulStage3):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(35)

        # random test input
        input_a = Int(32)(random.randint(0, 0xFFFFFFF))  # random input
        input_b = Int(32)(random.randint(0, 0xFFFFFFF))

        with Condition(cond):
            mulstage1.async_called(a=input_a, b=input_b, cnt=cnt[0])
            mulstage2.async_called(cnt=cnt[0])
            mulstage3.async_called(cnt=cnt[0], a=input_a, b=input_b)


def build_system():
    stage1_reg = RegArray(Int(64), 1)
    stage2_reg = RegArray(Int(64), 1)
    stage3_reg = RegArray(Int(64), 1)

    mulstage1 = MulStage1()
    mulstage1.build(stage1_reg)
    mulstage2 = MulStage2()
    mulstage2.build(stage1_reg, stage2_reg)
    mulstage3 = MulStage3()
    mulstage3.build(stage2_reg, stage3_reg)
    driver = Driver()
    driver.build(mulstage1, mulstage2, mulstage3)


def check_raw(raw):
    for i in raw.split('\n'):
        if 'Final' in i:
            line_toks = i.split()
            c = line_toks[-1]
            b = line_toks[-3]
            a = line_toks[-5]
            assert int(a) * int(b) == int(c)


def test_multiplier():
    run_test('multiplier_test', build_system, check_raw, verilog=True)


if __name__ == '__main__':
    test_multiplier()
