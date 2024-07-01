# import like others
import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils 


class Adder(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(explicit_fifo=True)
        self.a = Port(Int(32))
        self.b = Port(Int(32))

    @module.combinational
    def build(self):
        a = self.a.pop()
        b = self.b.pop()
        c = a + b
        log("adder: {} + {} = {}", a, b, c);

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        cnt[0] = v
        adder.async_called(a = v, b = v)


def test_explicit_pop():
    sys = SysBuilder('explicit_pop')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    print(sys)

    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    cnt = 0
    for i in raw.split('\n'):
        if f'[{adder.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'cnt: {cnt} != 100'


if __name__ == '__main__':
    test_explicit_pop()
