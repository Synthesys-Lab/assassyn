"""Basic Nested For-Loop FSM Example: Simple Accumulator

这个示例展示了嵌套for循环FSM模板的基础用法，实现一个简单的累加器。

计算公式：sum = 0 + 1 + 2 + ... + 99 = 4950

架构特点：
- OuterLoopFSM: 控制循环迭代（0到99）
- InnerComputeFSM: 累加数值（单周期计算）
- 握手协议: ready/valid/done信号
- 两个FSM都在Driver模块中实现，通过共享寄存器通信

重要说明：
本示例中inner FSM的所有状态都是单周期的（即每个状态在一个时钟周期内完成）。
这是Assassyn中避免调度冲突的关键设计原则。

预期输出：sum = 4950
"""

from assassyn.frontend import *
from assassyn.backend import *
from assassyn.ir.module import fsm
from assassyn import utils


class Driver(Module):
    """Driver模块：嵌套for循环FSM的主控模块

    本模块包含：
    1. 外层循环控制FSM：管理循环迭代
    2. 内层计算FSM：执行每次迭代的计算任务

    两个FSM通过握手协议（ready/valid/done信号）进行通信：
    - ready: 内层FSM准备好接收新数据
    - valid: 外层FSM发送有效数据
    - done: 内层FSM完成计算

    关键设计约束：
    在Assassyn中，当两个FSM都在同一个Module时，必须确保：
    - 所有FSM状态都是单周期的（无自循环）
    - 避免多周期计算状态（会导致"Already occupied"错误）
    """

    def __init__(self, loop_start=0, loop_end=100, loop_step=1):
        """初始化Driver模块

        Args:
            loop_start: 循环起始值（默认0）
            loop_end: 循环结束值（不包含，默认100）
            loop_step: 循环步长（默认1）
        """
        super().__init__(ports={})
        self.loop_start = loop_start
        self.loop_end = loop_end
        self.loop_step = loop_step

    @module.combinational
    def build(self):
        """构建嵌套FSM硬件逻辑

        本方法生成：
        1. 所有必需的寄存器
        2. 内层FSM（累加器）
        3. 外层FSM（循环控制器）
        """
        # ==================================================================
        # 共享寄存器定义
        # ==================================================================

        # 外层循环状态和控制寄存器
        outer_state = RegArray(Bits(2), 1, initializer=[0])  # 4个状态：init, wait_ready, execute, check_done
        loop_counter = RegArray(UInt(32), 1, initializer=[0])  # 循环计数器
        outer_valid = RegArray(Bits(1), 1, initializer=[0])  # 外层发送的valid信号

        # 内层FSM状态和结果寄存器
        inner_state = RegArray(Bits(2), 1, initializer=[0])  # 4个状态：idle, compute, done, reset
        result = RegArray(UInt(32), 1, initializer=[0])  # 累加结果

        # 握手协议信号
        inner_ready = RegArray(Bits(1), 1, initializer=[1])  # 内层准备好接收数据（初始为1）
        inner_done = RegArray(Bits(1), 1, initializer=[0])  # 内层计算完成标志

        # 迭代数据传递寄存器
        iteration_data = RegArray(UInt(32), 1, initializer=[0])  # 从外层传递给内层的数据

        # ==================================================================
        # 内层FSM：累加器
        #
        # 状态机流程：
        # 1. idle: 等待valid信号，发出ready信号
        # 2. compute: 执行累加操作（单周期）
        # 3. done: 发出done信号，通知外层完成
        # 4. reset: 复位握手信号，返回idle
        # ==================================================================

        # 定义内层FSM的转移条件
        inner_default = Bits(1)(1)  # 默认转移条件（总是真）
        inner_valid_high = outer_valid[0] == Bits(1)(1)  # 检测到valid信号

        # 内层FSM状态转移表
        inner_table = {
            "idle": {
                inner_valid_high: "compute",      # 收到valid -> 开始计算
                ~inner_valid_high: "idle"         # 未收到valid -> 保持idle
            },
            "compute": {inner_default: "done"},   # 计算完成 -> done（单周期）
            "done": {inner_default: "reset"},     # done -> reset
            "reset": {inner_default: "idle"},     # reset -> idle
        }

        # 内层FSM各状态的动作函数
        def inner_idle_action():
            """Idle状态：准备接收数据"""
            inner_ready[0] = Bits(1)(1)  # 发出ready信号
            inner_done[0] = Bits(1)(0)   # 清除done信号
            log("  InnerFSM: [IDLE] ready=1")

        def inner_compute_action():
            """Compute状态：执行累加计算（单周期）"""
            inner_ready[0] = Bits(1)(0)  # 取消ready
            inner_done[0] = Bits(1)(0)   # 还未done
            # 核心计算：累加
            result[0] = result[0] + iteration_data[0]
            log("  InnerFSM: [COMPUTE] iter={}, sum={}",
                iteration_data[0], result[0])

        def inner_done_action():
            """Done状态：发出完成信号"""
            inner_ready[0] = Bits(1)(0)  # 取消ready
            inner_done[0] = Bits(1)(1)   # 发出done信号
            log("  InnerFSM: [DONE] sum={}", result[0])

        def inner_reset_action():
            """Reset状态：复位握手信号"""
            inner_ready[0] = Bits(1)(0)  # 取消ready
            inner_done[0] = Bits(1)(0)   # 取消done
            log("  InnerFSM: [RESET]")

        # 将状态和动作函数关联
        inner_action_dict = {
            "idle": inner_idle_action,
            "compute": inner_compute_action,
            "done": inner_done_action,
            "reset": inner_reset_action,
        }

        # 生成内层FSM硬件逻辑
        inner_fsm_inst = fsm.FSM(inner_state, inner_table)
        inner_fsm_inst.generate(inner_action_dict)

        # ==================================================================
        # 外层FSM：循环控制器
        #
        # 状态机流程：
        # 1. init: 初始化循环参数
        # 2. wait_ready: 等待内层FSM准备好
        # 3. execute: 发送迭代数据给内层FSM
        # 4. check_done: 检查内层完成，决定继续循环或结束
        # ==================================================================

        # 循环参数（使用构造函数传入的值）
        loop_start = UInt(32)(self.loop_start)
        loop_end = UInt(32)(self.loop_end)
        loop_step = UInt(32)(self.loop_step)

        # 定义外层FSM的转移条件
        outer_default = Bits(1)(1)  # 默认转移
        ready_high = inner_ready[0] == Bits(1)(1)  # 内层ready
        done_high = inner_done[0] == Bits(1)(1)    # 内层done
        not_finished = loop_counter[0] < loop_end  # 循环未完成
        finished = loop_counter[0] >= loop_end     # 循环已完成

        # 外层FSM状态转移表
        outer_table = {
            "init": {outer_default: "wait_ready"},
            "wait_ready": {
                ready_high: "execute",        # ready=1 -> 发送数据
                ~ready_high: "wait_ready"     # ready=0 -> 继续等待
            },
            "execute": {outer_default: "check_done"},
            "check_done": {
                done_high & not_finished: "wait_ready",  # done=1且未完成 -> 下一次迭代
                done_high & finished: "check_done",      # done=1且已完成 -> 结束
                ~done_high: "check_done",                # done=0 -> 继续等待done
            },
        }

        # 外层FSM各状态的动作函数
        def outer_init_action():
            """Init状态：初始化循环参数"""
            loop_counter[0] = loop_start
            outer_valid[0] = Bits(1)(0)
            log("OuterFSM: [INIT] start={}, end={}, step={}",
                loop_start, loop_end, loop_step)

        def outer_wait_ready_action():
            """Wait_Ready状态：等待内层准备好"""
            outer_valid[0] = Bits(1)(0)  # 清除valid
            log("OuterFSM: [WAIT_READY] counter={}, ready={}",
                loop_counter[0], inner_ready[0])

        def outer_execute_action():
            """Execute状态：发送迭代数据"""
            iteration_data[0] = loop_counter[0]  # 传递当前迭代值
            outer_valid[0] = Bits(1)(1)          # 发出valid信号
            log("OuterFSM: [EXECUTE] sending iter={}", loop_counter[0])

        def outer_check_done_action():
            """Check_Done状态：检查完成并更新计数器"""
            outer_valid[0] = Bits(1)(0)  # 清除valid

            # 如果内层完成，递增计数器
            with Condition(inner_done[0] == Bits(1)(1)):
                loop_counter[0] = loop_counter[0] + loop_step
                log("OuterFSM: [CHECK_DONE] iter {} done, next={}",
                    loop_counter[0] - loop_step, loop_counter[0])

            # 如果循环结束，调用finish()
            with Condition(loop_counter[0] >= loop_end):
                log("OuterFSM: [DONE] Loop complete! Final sum={}", result[0])
                finish()

        # 将状态和动作函数关联
        outer_action_dict = {
            "init": outer_init_action,
            "wait_ready": outer_wait_ready_action,
            "execute": outer_execute_action,
            "check_done": outer_check_done_action,
        }

        # 生成外层FSM硬件逻辑
        outer_fsm_inst = fsm.FSM(outer_state, outer_table)
        outer_fsm_inst.generate(outer_action_dict)


def test_basic_example(loop_start=0, loop_end=100, loop_step=1, expected_sum=4950):
    """构建并运行基础累加器示例

    Args:
        loop_start: 循环起始值
        loop_end: 循环结束值（不包含）
        loop_step: 循环步长
        expected_sum: 预期的累加结果

    Returns:
        bool: 测试是否通过
    """
    print("=" * 60)
    print("Basic Nested For-Loop FSM Example: Accumulator")
    print("=" * 60)
    print(f"Computing: sum = {loop_start} + {loop_start+loop_step} + ... + {loop_end-loop_step}")
    print(f"Expected result: {expected_sum}")
    print("=" * 60)

    # 构建系统
    sys = SysBuilder('basic_loop_fsm')
    with sys:
        driver = Driver(loop_start=loop_start, loop_end=loop_end, loop_step=loop_step)
        driver.build()

    print("\nSystem built successfully")

    # 配置和elaboration
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=3000,
        idle_threshold=100,
    )

    print("Elaborating system...")
    simulator_path, verilog_path = elaborate(sys, **conf)
    print(f"Simulator: {simulator_path}")

    # 运行仿真
    print("\n" + "=" * 60)
    print("Running Simulation...")
    print("=" * 60)
    raw = utils.run_simulator(simulator_path)

    # 显示输出的最后几行
    print("\n" + "=" * 60)
    print("Simulation Output (last 30 lines):")
    print("=" * 60)
    lines = raw.split('\n')
    for line in lines[-30:]:
        if line.strip():
            print(line)

    # 验证结果
    print("\n" + "=" * 60)
    print("Verification:")
    print("=" * 60)

    test_passed = False
    for line in lines:
        if "Final sum=" in line:
            parts = line.split("Final sum=")
            if len(parts) > 1:
                sum_str = parts[1].strip()
                try:
                    final_sum = int(sum_str)
                    if final_sum == expected_sum:
                        print(f"✅ SUCCESS: sum = {final_sum}")
                        test_passed = True
                    else:
                        print(f"❌ FAILED: sum = {final_sum} (expected {expected_sum})")
                except ValueError:
                    print(f"⚠️  Could not parse: {sum_str}")
                break
    else:
        print("⚠️  Final sum not found in output")

    # Verilator仿真
    if verilog_path and utils.has_verilator():
        print("\n" + "=" * 60)
        print("Running Verilator...")
        print("=" * 60)
        raw_v = utils.run_verilator(verilog_path)
        for line in raw_v.split('\n'):
            if "Final sum=" in line:
                parts = line.split("Final sum=")
                if len(parts) > 1:
                    try:
                        final_sum = int(parts[1].strip())
                        if final_sum == expected_sum:
                            print(f"✅ Verilator SUCCESS: sum = {final_sum}")
                        else:
                            print(f"❌ Verilator FAILED: sum = {final_sum}")
                    except ValueError:
                        pass
                    break

    print("\n" + "=" * 60)
    print("Test Complete")
    print("=" * 60)

    return test_passed


if __name__ == '__main__':
    # 默认测试：sum(0..99) = 4950
    success = test_basic_example()

    # 也可以测试其他范围，例如：
    # test_basic_example(loop_start=1, loop_end=11, loop_step=1, expected_sum=55)  # sum(1..10) = 55

    import sys
    sys.exit(0 if success else 1)
