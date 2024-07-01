from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind

class Sub(Module):

    @module.constructor
    def __init__(self):
        self.sub_a = Port(Int(32))
        self.sub_b = Port(Int(32))

    @module.combinational
    def build(self):
        c = self.sub_a - self.sub_b
        log("Subtractor: {} - {} = {}", self.sub_a, self.sub_b, c)

class Lhs(Module):

    @module.constructor
    def __init__(self):
        self.lhs_a = Port(Int(32))

    @module.combinational
    def build(self, sub: Sub):
        bound = sub.bind(sub_a = self.lhs_a)
        return bound

class Rhs(Module):

    @module.constructor
    def __init__(self):
        self.rhs_b = Port(Int(32))

    @module.combinational
    def build(self, sub: Bind):
        sub.async_called(sub_b = self.rhs_b)

class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self, lhs: Lhs, rhs: Rhs):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        v = cnt[0] * cnt[0]

        lhs.async_called(lhs_a = v[0: 31].bitcast(Int(32)))
        rhs.async_called(rhs_b = cnt[0])

def test_bind():
    sys =  SysBuilder('bind')
    with sys:
        sub = Sub()
        sub.build()

        lhs = Lhs()
        aa_lhs = lhs.build(sub)

        rhs = Rhs()
        rhs.build(aa_lhs)

        driver = Driver()
        driver.build(lhs, rhs)

    print(sys)
    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    cnt = 0
    for i in raw.split('\n'):
        if f'[{sub.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(b) - int(a) == int(c)
            cnt += 1
    assert cnt == 100 - 1, f'cnt: {cnt} != 100'

if __name__ == '__main__':
    test_bind()
