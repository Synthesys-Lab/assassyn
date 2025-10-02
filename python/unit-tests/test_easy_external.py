from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.ir.module.external import ExternalSV


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, lhs: Module, rhs: Module):
        cnt = RegArray(UInt(32), 1)
        v = cnt[0]
        cnt[0] = cnt[0] + UInt(32)(1)
        lhs.async_called(data=v)
        rhs.async_called(data=v)


class ForwardData(Module):
    def __init__(self):
        super().__init__(
            ports={'data': Port(UInt(32))},
        )

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        return data

class ExternalAdder(ExternalSV):
    '''External SystemVerilog adder module.'''

    def __init__(self, **in_wire_connections):
        super().__init__(
            file_path="python/unit-tests/resources/adder.sv",
            module_name="adder",
            in_wires={
                'a': UInt(32),
                'b': UInt(32),
            },
            out_wires={
                'c': UInt(32),
            },
            **in_wire_connections
        )

    def __getattr__(self, name):
        # Allow accessing output wires as attributes
        if hasattr(self, 'out_wires') and name in self.out_wires:
            return self.out_wires[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")


class Adder(Downstream):

    def __init__(self):
        super().__init__()

    @downstream.combinational
    def build(self, a: Value, b: Value, ext_adder: ExternalAdder):
        #here we assumed user explicitly know the direction of the external module ports
        a = a.optional(UInt(32)(1))
        b = b.optional(UInt(32)(1))

        # Instantiate the external adder module and capture its single output
        c = ext_adder.in_assign(a=a, b=b)

        log("downstream: {} + {} = {}", a, b, c)


def test_easy_external():
    sys = SysBuilder('easy_external')
    with sys:
        driver = Driver()
        lhs = ForwardData()
        rhs = ForwardData()
        a = lhs.build()
        b = rhs.build()

        ext_adder = ExternalAdder()
        adder = Adder()

        driver.build(lhs, rhs)
        adder.build(a, b, ext_adder)

    config = {
        'verilog': utils.has_verilator(),
        'simulator': False,
        'sim_threshold': 100,
        'idle_threshold': 100
    }

    simulator_path, verilator_path = elaborate(sys, **config)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        assert "downstream:" in raw, "Expected log output not found"


if __name__ == '__main__':
    test_easy_external()
