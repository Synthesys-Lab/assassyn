from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import assassyn


class FSM_m(Module):

    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
            },
        )

    @module.combinational
    def build(self,state:Array):
        a= self.pop_all_ports(True)

        temp = RegArray(Int(32), 1)

        with Condition(state[0] == UInt(2)(0)):
            temp[0] = a
            state[0] = UInt(2)(1)
        with Condition(state[0] == UInt(2)(1)):
            with Condition(a[0:1] == UInt(2)(0)):
                state[0] = UInt(2)(2)
        with Condition(state[0] == UInt(2)(2)):
            state[0] = UInt(2)(3)
        with Condition(state[0] == UInt(2)(3)):
            temp[0] = (temp[0] * Int(32)(2)).bitcast(Int(32))
            state[0] = UInt(2)(0)

        log("state: {} | a: {} |  temp: {} ", state[0] , a , temp[0])
        

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, adder: FSM_m):

        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            adder.async_called(a = cnt[0])




def test_fsm_gold():
    sys = SysBuilder('FSM_gold')
    with sys:

        state = RegArray(UInt(2), 1 , initializer=[0])

        adder1 = FSM_m()
        adder1.build(state)

        driver = Driver()
        driver.build(adder1)

    print(sys)

    config = assassyn.backend.config(
            verilog=utils.has_verilator(),
            sim_threshold=200,
            idle_threshold=200,
            random=True)

    simulator_path, verilator_path  = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    if verilator_path:
        raw = utils.run_verilator(verilator_path)


if __name__ == '__main__':
    test_fsm_gold()
