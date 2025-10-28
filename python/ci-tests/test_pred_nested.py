from assassyn.frontend import *
from assassyn.test import run_test


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)

        c1 = cnt[0] < Int(32)(20)
        c2 = (cnt[0][0:0]) == Bits(1)(0)

        push_condition(c1)
        push_condition(c2)
        print('here1')
        pred = get_pred()
        print('here2')
        # Log pred value and counter every cycle to validate get_pred semantics
        log("pred={} cnt={}", pred, cnt[0])
        print('here3')
        pop_condition()
        pop_condition()


def check_raw(raw):
    # Expect pred==1 for even numbers under 20: 0,2,...,18
    seen = set()
    for i in raw.split("\n"):
        if "pred=" in i and "cnt=" in i:
            toks = i.strip().split()
            try:
                pred_str = [t for t in toks if t.startswith('pred=')][0]
                cnt_str = [t for t in toks if t.startswith('cnt=')][0]
                pred_val = int(pred_str.split('=')[1])
                cnt_val = int(cnt_str.split('=')[1])
            except Exception:
                continue
            if pred_val == 1:
                seen.add(cnt_val)
    expected = set(range(0, 20, 2))
    assert expected.issubset(seen), f"missing values: {sorted(expected - seen)}"


def test_pred_nested():
    def top():
        driver = Driver()
        driver.build()

    run_test("pred_nested", top, check_raw, sim_threshold=50, idle_threshold=50, random=True, verilog=True)


if __name__ == "__main__":
    test_pred_nested()

