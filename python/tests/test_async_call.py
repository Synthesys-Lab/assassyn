from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Adder(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))
        self.b = Port(Int(32))

    @module.combinational
    def build(self):
        c = self.a + self.b
        log("Adder: {} + {} = {}", self.a, self.b, c)

class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        adder.async_called(a = cnt[0], b = cnt[0])

def test_async_call():
    sys = SysBuilder('async_call')
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
        if f'[{adder.as_operand().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'{cnt} != 100'


if __name__ == '__main__':
    test_async_call()
