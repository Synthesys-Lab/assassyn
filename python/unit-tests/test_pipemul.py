import assassyn
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

class ExternalMultiplier(ExternalSV):
    '''External SystemVerilog multiplier module.'''

    def __init__(self, **in_wire_connections):
        super().__init__(
            file_path="python/unit-tests/resources/mul_pipe_simple.sv",
            module_name="mul_pipe_simple",
            has_clock=True,
            has_reset=True,
            in_wires={
                'a': UInt(32),
                'b': UInt(32),
                'in_valid': Bits(1),
            },
            out_wires={
                'p': UInt(64),
                'out_valid': Bits(1),
            },
            **in_wire_connections
        )

    def __getattr__(self, name):
        # Allow accessing output wires as attributes
        if hasattr(self, 'out_wires') and name in self.out_wires:
            return self.out_wires[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")

class Sink(Module):

    def __init__(self):
        super().__init__(
            ports={'data': Port(UInt(32*2))},
        )

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        log("Sink: {}", data)

class Wrapper(Downstream):

    def __init__(self):
        super().__init__()

    @downstream.combinational
    def build(self, a: Value, b: Value, ext_mul: ExternalMultiplier , sink: Sink):
        #here we assumed user explicitly know the direction of the external module ports
        a = a.optional(UInt(32)(1))
        b = b.optional(UInt(32)(1))
        in_valid = Bits(1)(1)  # Always valid input
        out_ready = Bits(1)(0)

        # Instantiate the external multiplier module and capture outputs
        p, out_ready = ext_mul.in_assign(a=a, b=b, in_valid=in_valid)

        with Condition(out_ready):
            sink.async_called(data=p)

        log("downstream: {} * {} ", a, b)


def test_pipemul_external():
    sys = SysBuilder('pipemul_external')
    with sys:
        driver = Driver()
        lhs = ForwardData()
        rhs = ForwardData()
        a = lhs.build()
        b = rhs.build()

        ext_mul = ExternalMultiplier()
        wrapper = Wrapper()
        sink = Sink()
        sink.build()

        driver.build(lhs, rhs)
        wrapper.build(a, b, ext_mul, sink)

    print(sys)

    config = assassyn.backend.config(
            verilog = utils.has_verilator(),  # Force verilog generation
            simulator=False,
            sim_threshold=100,
            idle_threshold=100)

    simulator_path, verilator_path = elaborate(sys, **config )


    if verilator_path:
        raw = utils.run_verilator(verilator_path)


if __name__ == '__main__':
    test_pipemul_external()
