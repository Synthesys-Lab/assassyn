from assassyn.frontend import *
from assassyn.test import run_test

class Adder2(Module):

    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
            },
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a * b

        d = a + b + c
        log("combi: {} + {} + {}*{} = {} ", a, b, a, b, d)

        return d

class Adder1(Module):

    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
                'c': Port(Int(32)),
            },
        )

    @module.combinational
    def build(self,adder: Adder2,is_gold = False):
        a, b , c= self.pop_all_ports(True)
        e = a * b
        with Condition(e < Int(64)(100)):
            d = a + b + e
            if not is_gold:
                barrier(d)
            f = d * c
            adder.async_called(a = f[0:31].bitcast(Int(32)), b = d[0:31].bitcast(Int(32)))

        return f


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, adder: Adder1):

        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cnt_div2_temp = cnt[0] + Int(32)(1)
        cnt_div2 = Int(32)(0)
        cnt_div2 = cnt[0][0:0].select(cnt[0], cnt_div2_temp)

        cond = cnt[0] < Int(32)(100)

        with Condition(cond):
            adder.async_called(a = cnt_div2, b = cnt_div2 , c = cnt_div2)


def build_system(is_gold):
    def top():
        adder2 = Adder2()
        res = adder2.build()

        adder1 = Adder1()
        d = adder1.build(adder2, is_gold)

        driver = Driver()
        driver.build(adder1)
    return top

def checker(raw):
    # Checker just validates that the test runs without errors
    pass

def test_block_barrier():
    run_test(
        'Comb_block_barrier',
        build_system(False),
        checker,
        sim_threshold=200,
        idle_threshold=200,
        random=True
    )

def test_block_barrier_gold():
    run_test(
        'Comb_block_barrier_gold',
        build_system(True),
        checker,
        sim_threshold=200,
        idle_threshold=200,
        random=True
    )

if __name__ == '__main__':
    test_block_barrier()
    test_block_barrier_gold()


