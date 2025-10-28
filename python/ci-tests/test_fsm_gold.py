from assassyn.frontend import *
from assassyn.test import run_test


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
            (temp & self)[0] <= a
            (state & self)[0] <= UInt(2)(1)

        with Condition(state[0] == UInt(2)(1)):
            with Condition(a[0:1] == UInt(2)(0)):
                (state & self)[0] <= UInt(2)(2)

        with Condition(state[0] == UInt(2)(2)):
            (state & self)[0] <= UInt(2)(3)

        with Condition(state[0] == UInt(2)(3)):
            (temp & self)[0] <= (temp[0] * Int(32)(2)).bitcast(Int(32))
            (state & self)[0] <= UInt(2)(0)

        log("state: {} | a: {} |  temp: {} ", state[0] , a , temp[0])


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, adder: FSM_m):

        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            adder.async_called(a = cnt[0])




def build_system():
    state = RegArray(UInt(2), 1 , initializer=[0])

    adder1 = FSM_m()
    adder1.build(state)

    driver = Driver()
    driver.build(adder1)


def checker(raw):
    # Basic validation that simulation ran
    assert raw is not None, "Simulator output is None"


def test_fsm_gold():
    run_test(
        name='FSM_gold',
        top=build_system,
        checker=checker,
        sim_threshold=200,
        idle_threshold=200,
        random=True,
        verilog=True
    )


if __name__ == '__main__':
    test_fsm_gold()
