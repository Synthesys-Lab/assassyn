# Nested For-Loop FSM Template Specification

> **Author**: Claude Code
> **Date**: 2025-11-30
> **Version**: 1.0

---

## 1. Introduction

### 1.1 Motivation

在硬件设计中，循环结构（for-loop）是常见的控制流模式。许多硬件加速器需要实现嵌套循环，其中：
- **外层循环**负责迭代控制（循环计数器管理）
- **内层循环/计算单元**执行每次迭代的具体计算（可能需要多个时钟周期）

现有问题：
1. 循环控制逻辑与计算逻辑耦合，难以复用
2. 多周期计算需要手动管理握手信号，容易出错
3. 状态机代码冗长，可读性差

### 1.2 Design Goals

本规范定义一个**可复用的嵌套循环FSM模板**，实现以下目标：

1. **模块化（Modularity）**: 分离循环控制逻辑和计算逻辑
2. **可复用性（Reusability）**: 提供模板，适用于不同的循环模式
3. **清晰性（Clarity）**: 使用 Assassyn FSM 抽象，声明式定义状态机
4. **高效性（Efficiency）**: 通过握手协议最小化周期开销
5. **可扩展性（Extensibility）**: 支持用户自定义计算逻辑

### 1.3 Scope

本规范涵盖：
- 双层FSM架构（外层循环控制器 + 内层计算单元）
- 握手协议（ready/valid/done 信号）
- 接口规范和行为定义
- 使用示例

本规范不涵盖：
- 三层及以上嵌套循环（作为未来扩展）
- 并行内层FSM（作为未来扩展）
- 动态循环边界（作为未来扩展）

---

## 2. Architecture Overview

### 2.1 Two-Level FSM Hierarchy

系统由两个协同工作的有限状态机组成：

```
┌─────────────────────────────────────────────────────────────┐
│                    OuterLoopFSM (外层循环控制器)              │
│  ┌──────┐   ┌────────────┐   ┌─────────┐   ┌────────────┐  │
│  │ init │──>│ wait_ready │──>│ execute │──>│ check_done │  │
│  └──────┘   └────────────┘   └─────────┘   └────────────┘  │
│       │            │               │              │         │
│       │            │               │              └─────┐   │
│       │            │               │                    │   │
└───────┼────────────┼───────────────┼────────────────────┼───┘
        │            │               │                    │
        │      inner_ready     outer_valid          inner_done
        │            │               │                    │
┌───────┼────────────┼───────────────┼────────────────────┼───┐
│       │            │               │                    │   │
│       ▼            ▼               ▼                    ▼   │
│  ┌──────┐   ┌─────────┐   ┌──────┐   ┌───────┐            │
│  │ idle │<──│  reset  │<──│ done │<──│compute│            │
│  └──────┘   └─────────┘   └──────┘   └───────┘            │
│                 InnerComputeFSM (内层计算单元)              │
└─────────────────────────────────────────────────────────────┘
```

**外层FSM (OuterLoopFSM)**:
- **init**: 初始化循环计数器
- **wait_ready**: 等待内层FSM就绪
- **execute**: 触发内层FSM计算
- **check_done**: 检查是否继续循环或退出

**内层FSM (InnerComputeFSM)**:
- **idle**: 就绪状态，等待新的迭代
- **compute**: 执行计算（可能多周期）
- **done**: 计算完成，发送完成信号
- **reset**: 重置状态，准备下一次迭代

### 2.2 Handshake Protocol

使用三个握手信号协调两个FSM：

| 信号名 | 方向 | 位宽 | 描述 |
|--------|------|------|------|
| `inner_ready` | Inner → Outer | 1 bit | 内层FSM就绪信号（可接受新迭代） |
| `outer_valid` | Outer → Inner | 1 bit | 外层FSM有效信号（发送新迭代数据） |
| `inner_done` | Inner → Outer | 1 bit | 内层FSM完成信号（计算完成） |

### 2.3 Protocol Timing Diagram

```
Cycle:     0    1    2    3    4    5    6    7    8    9   10   11   12
           ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
clock      │    │    │    │    │    │    │    │    │    │    │    │    │
           └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘

OuterFSM:  init wait exec chk  wait exec chk  wait exec chk  wait ...
           ───┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐
InnerFSM:  idle │  │cpt│cpt│done│rset│idle│  │cpt│cpt│done│rset│idle
           ────┘ └──┘ └──┘ └──┘ └──┘ └──┘ └──┘ └──┘ └──┘ └──┘ └──┘

inner_ready: ────┐         ┌─────┐         ┌─────┐         ┌──────
                 └─────────┘     └─────────┘     └─────────┘

outer_valid: ────┐    ┌──────────┐    ┌──────────┐    ┌──────────
                 └────┘          └────┘          └────┘

inner_done:  ─────────────┐    ┌──────────┐    ┌──────────┐    ┌──
                          └────┘          └────┘          └────┘

loop_counter: 0    0    0    0    1    1    1    2    2    2    3
              ────────────────────────────────────────────────────
```

**协议流程**：
1. **Cycle 0**: 内层FSM处于idle状态，拉高 `inner_ready`
2. **Cycle 1**: 外层FSM检测到 `inner_ready`，拉高 `outer_valid`，发送迭代数据
3. **Cycle 2**: 内层FSM接收数据，拉低 `inner_ready`，进入compute状态
4. **Cycle 3-4**: 内层FSM执行多周期计算
5. **Cycle 5**: 内层FSM完成计算，拉高 `inner_done`
6. **Cycle 6**: 外层FSM检测到 `inner_done`，递增计数器
7. **Cycle 7**: 内层FSM重置，外层FSM等待下一次 `inner_ready`
8. 重复步骤 1-7

---

## 3. Outer Loop FSM Specification

### 3.1 State Machine Definition

**状态编码** (2 bits):
- `00`: init (初始化)
- `01`: wait_ready (等待就绪)
- `10`: execute (执行)
- `11`: check_done (检查完成)

**状态转移表**:

```python
outer_transition_table = {
    "init": {
        default: "wait_ready"
    },
    "wait_ready": {
        inner_ready == 1: "execute",
        inner_ready == 0: "wait_ready"
    },
    "execute": {
        default: "check_done"
    },
    "check_done": {
        (inner_done == 1) & (counter < loop_end): "wait_ready",
        (inner_done == 1) & (counter >= loop_end): "finish",
        inner_done == 0: "check_done"
    }
}
```

### 3.2 Interface

**输入参数**:
- `inner_fsm: Module` - 内层计算FSM模块引用
- `loop_start: Value` - 循环起始值
- `loop_end: Value` - 循环结束值（不包含）
- `loop_step: Value` - 循环步长

**输出**:
- `loop_counter: RegArray` - 当前迭代计数器值
- `loop_done: RegArray` - 循环完成信号

**内部寄存器**:
- `outer_state: RegArray(Bits(2), 1)` - 状态寄存器
- `loop_counter: RegArray(UInt(counter_width), 1)` - 循环计数器
- `outer_valid: RegArray(Bits(1), 1)` - 有效信号寄存器

### 3.3 Behavior Specification

#### State: init
**目的**: 初始化循环计数器

**行为**:
```python
def init_action():
    loop_counter[0] = loop_start
    outer_valid[0] = Bits(1)(0)
    log("OuterFSM: Initializing loop, start={}", loop_start)
```

**出口条件**: 无条件转移到 `wait_ready`

#### State: wait_ready
**目的**: 等待内层FSM就绪

**行为**:
```python
def wait_ready_action():
    outer_valid[0] = Bits(1)(0)
    log("OuterFSM: Waiting for inner FSM ready, counter={}", loop_counter[0])
```

**出口条件**:
- `inner_ready == 1` → 转移到 `execute`
- `inner_ready == 0` → 保持在 `wait_ready`

#### State: execute
**目的**: 发送迭代数据到内层FSM

**行为**:
```python
def execute_action():
    outer_valid[0] = Bits(1)(1)
    # 通过 async_called 传递迭代数据到内层FSM
    inner_fsm.async_called(
        iteration=loop_counter[0],
        valid=outer_valid[0]
    )
    log("OuterFSM: Executing iteration {}", loop_counter[0])
```

**出口条件**: 无条件转移到 `check_done`

#### State: check_done
**目的**: 检查内层FSM是否完成，决定继续或结束循环

**行为**:
```python
def check_done_action():
    outer_valid[0] = Bits(1)(0)
    with Condition(inner_done[0] == Bits(1)(1)):
        loop_counter[0] = loop_counter[0] + loop_step
        log("OuterFSM: Iteration {} done, incrementing", loop_counter[0])
```

**出口条件**:
- `(inner_done == 1) & (counter < loop_end)` → 转移到 `wait_ready`
- `(inner_done == 1) & (counter >= loop_end)` → 转移到 `finish`
- `inner_done == 0` → 保持在 `check_done`

---

## 4. Inner Compute FSM Specification

### 4.1 State Machine Definition

**状态编码** (2 bits):
- `00`: idle (空闲就绪)
- `01`: compute (计算中)
- `10`: done (完成)
- `11`: reset (重置)

**状态转移表**:

```python
inner_transition_table = {
    "idle": {
        valid == 1: "compute",
        valid == 0: "idle"
    },
    "compute": {
        compute_complete: "done",
        ~compute_complete: "compute"
    },
    "done": {
        default: "reset"
    },
    "reset": {
        default: "idle"
    }
}
```

### 4.2 Interface

**输入端口** (Ports):
- `iteration: Port(UInt(data_width))` - 当前迭代数据
- `valid: Port(Bits(1))` - 外层FSM有效信号

**输入参数**:
- `ready_out: RegArray(Bits(1), 1)` - 就绪信号输出寄存器（共享）
- `done_out: RegArray(Bits(1), 1)` - 完成信号输出寄存器（共享）
- `compute_func: callable (optional)` - 用户自定义计算函数

**输出**:
- `result: RegArray` - 计算结果寄存器

**内部寄存器**:
- `inner_state: RegArray(Bits(2), 1)` - 状态寄存器
- `result_reg: RegArray(UInt(data_width), 1)` - 结果寄存器
- `compute_cycles: RegArray(UInt(8), 1)` - 计算周期计数器

### 4.3 Behavior Specification

#### State: idle
**目的**: 就绪状态，等待新的迭代

**行为**:
```python
def idle_action():
    ready_out[0] = Bits(1)(1)       # 拉高ready信号
    done_out[0] = Bits(1)(0)        # 拉低done信号
    compute_cycles[0] = UInt(8)(0)  # 重置计算周期计数器
    log("InnerFSM: Idle, ready for next iteration")
```

**出口条件**:
- `valid == 1` → 转移到 `compute`
- `valid == 0` → 保持在 `idle`

#### State: compute
**目的**: 执行用户定义的计算逻辑

**行为**:
```python
def compute_action():
    ready_out[0] = Bits(1)(0)   # 拉低ready信号（忙碌）
    done_out[0] = Bits(1)(0)    # 保持done为低

    # 用户自定义计算逻辑
    if compute_func:
        result_reg[0] = compute_func(iteration, compute_cycles[0])
    else:
        # 默认计算：累加
        result_reg[0] = result_reg[0] + iteration

    compute_cycles[0] = compute_cycles[0] + UInt(8)(1)
    log("InnerFSM: Computing iteration={}, cycle={}",
        iteration, compute_cycles[0])
```

**出口条件**:
- `compute_complete` (用户定义) → 转移到 `done`
- `~compute_complete` → 保持在 `compute`

**计算完成条件示例**:
```python
# 示例1: 固定周期数
compute_complete = (compute_cycles[0] >= UInt(8)(10))

# 示例2: 基于数据依赖
compute_complete = (result_reg[0] > threshold)
```

#### State: done
**目的**: 发送完成信号

**行为**:
```python
def done_action():
    ready_out[0] = Bits(1)(0)   # 保持ready为低
    done_out[0] = Bits(1)(1)    # 拉高done信号
    log("InnerFSM: Computation done, result={}", result_reg[0])
```

**出口条件**: 无条件转移到 `reset`

#### State: reset
**目的**: 重置状态，准备下一次迭代

**行为**:
```python
def reset_action():
    ready_out[0] = Bits(1)(0)   # 拉低ready（重置中）
    done_out[0] = Bits(1)(0)    # 拉低done
    log("InnerFSM: Resetting for next iteration")
```

**出口条件**: 无条件转移到 `idle`

---

## 5. Handshake Protocol Specification

### 5.1 Signal Definitions

| 信号名 | 类型 | 方向 | 位宽 | 有效电平 | 描述 |
|--------|------|------|------|----------|------|
| `inner_ready` | 输出（Inner）<br>输入（Outer） | Inner → Outer | 1 bit | 高有效 | 内层FSM处于idle状态，可接受新迭代 |
| `outer_valid` | 输出（Outer）<br>输入（Inner） | Outer → Inner | 1 bit | 高有效 | 外层FSM发送有效的迭代数据 |
| `inner_done` | 输出（Inner）<br>输入（Outer） | Inner → Outer | 1 bit | 高有效 | 内层FSM完成当前迭代的计算 |

### 5.2 Protocol Sequence

**正常操作序列**:

1. **初始状态**
   - Inner FSM: `idle` 状态
   - `inner_ready = 1`, `outer_valid = 0`, `inner_done = 0`

2. **握手建立**
   - Outer FSM 检测到 `inner_ready == 1`
   - Outer FSM 设置 `outer_valid = 1` 并发送迭代数据

3. **数据传输**
   - Inner FSM 在下一个周期采样 `outer_valid` 和迭代数据
   - Inner FSM 设置 `inner_ready = 0`（表示忙碌）
   - Inner FSM 进入 `compute` 状态

4. **计算阶段**
   - Inner FSM 保持在 `compute` 状态（可能多周期）
   - `inner_ready = 0`, `outer_valid = 0`, `inner_done = 0`

5. **完成通知**
   - Inner FSM 完成计算，进入 `done` 状态
   - Inner FSM 设置 `inner_done = 1`

6. **握手释放**
   - Outer FSM 检测到 `inner_done == 1`
   - Outer FSM 递增循环计数器
   - Inner FSM 进入 `reset` 状态，设置 `inner_done = 0`
   - Inner FSM 返回 `idle` 状态，设置 `inner_ready = 1`

7. **循环继续**
   - 如果 `loop_counter < loop_end`，返回步骤 2
   - 否则，循环结束

### 5.3 Timing Requirements

**建立时间（Setup Time）**:
- `outer_valid` 必须在 Inner FSM 采样前至少保持 1 个周期

**保持时间（Hold Time）**:
- `inner_ready` 必须在 Outer FSM 检测后保持稳定
- `inner_done` 必须保持至少 1 个周期，直到 Outer FSM 确认

**响应延迟（Response Latency）**:
- Outer FSM 检测到 `inner_ready` 后，在下一个周期拉高 `outer_valid`
- Inner FSM 检测到 `outer_valid` 后，在同一周期拉低 `inner_ready`

---

## 6. Usage Examples

### 6.1 Basic Loop Example

**场景**: 简单的累加循环，计算 sum = 0 + 1 + 2 + ... + 99

```python
from assassyn.frontend import *
from assassyn.backend import *
from assassyn.ir.module import fsm

# 内层FSM：单周期累加
class SimpleAccumulator(Module):
    def __init__(self):
        super().__init__(
            ports={
                'iteration': Port(UInt(32)),
                'valid': Port(Bits(1)),
            }
        )

    @module.combinational
    def build(self, ready_out, done_out):
        iteration, valid = self.pop_all_ports(True)

        # 状态和结果寄存器
        inner_state = RegArray(Bits(2), 1, initializer=[0])
        result_reg = RegArray(UInt(32), 1, initializer=[0])

        # 转移条件
        default = Bits(1)(1)
        valid_high = valid == Bits(1)(1)

        # 转移表
        inner_table = {
            "idle": {valid_high: "compute", ~valid_high: "idle"},
            "compute": {default: "done"},
            "done": {default: "reset"},
            "reset": {default: "idle"},
        }

        # 状态动作
        def idle_action():
            ready_out[0] = Bits(1)(1)
            done_out[0] = Bits(1)(0)
            log("InnerFSM: Idle, ready for iteration")

        def compute_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(0)
            result_reg[0] = result_reg[0] + iteration
            log("InnerFSM: Accumulating iteration={}, sum={}",
                iteration, result_reg[0])

        def done_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(1)
            log("InnerFSM: Done, current sum={}", result_reg[0])

        def reset_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(0)

        action_dict = {
            "idle": idle_action,
            "compute": compute_action,
            "done": done_action,
            "reset": reset_action,
        }

        # 生成FSM
        inner_fsm = fsm.FSM(inner_state, inner_table)
        inner_fsm.generate(action_dict)

        return result_reg

# 外层FSM：循环控制器
class OuterLoopController(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, inner_fsm, loop_start, loop_end, loop_step):
        # 状态寄存器
        outer_state = RegArray(Bits(2), 1, initializer=[0])
        loop_counter = RegArray(UInt(32), 1, initializer=[0])
        outer_valid = RegArray(Bits(1), 1, initializer=[0])
        loop_done_reg = RegArray(Bits(1), 1, initializer=[0])

        # 握手信号（与内层FSM共享）
        inner_ready = RegArray(Bits(1), 1, initializer=[1])
        inner_done = RegArray(Bits(1), 1, initializer=[0])

        # 调用内层FSM
        result = inner_fsm.build(inner_ready, inner_done)

        # 转移条件
        default = Bits(1)(1)
        ready_high = inner_ready[0] == Bits(1)(1)
        done_high = inner_done[0] == Bits(1)(1)
        not_finished = loop_counter[0] < loop_end
        finished = loop_counter[0] >= loop_end

        # 转移表
        outer_table = {
            "init": {default: "wait_ready"},
            "wait_ready": {ready_high: "execute", ~ready_high: "wait_ready"},
            "execute": {default: "check_done"},
            "check_done": {done_high & not_finished: "wait_ready",
                          done_high & finished: "finish"},
        }

        # 状态动作
        def init_action():
            loop_counter[0] = loop_start
            outer_valid[0] = Bits(1)(0)
            log("OuterFSM: Initializing, start={}, end={}", loop_start, loop_end)

        def wait_ready_action():
            outer_valid[0] = Bits(1)(0)
            log("OuterFSM: Waiting for ready, counter={}", loop_counter[0])

        def execute_action():
            outer_valid[0] = Bits(1)(1)
            inner_fsm.async_called(
                iteration=loop_counter[0],
                valid=outer_valid[0]
            )
            log("OuterFSM: Executing iteration {}", loop_counter[0])

        def check_done_action():
            outer_valid[0] = Bits(1)(0)
            with Condition(inner_done[0] == Bits(1)(1)):
                loop_counter[0] = loop_counter[0] + loop_step
                log("OuterFSM: Iteration {} done", loop_counter[0] - loop_step)

        def finish_action():
            loop_done_reg[0] = Bits(1)(1)
            log("OuterFSM: Loop completed! Final result={}", result[0])
            finish()

        action_dict = {
            "init": init_action,
            "wait_ready": wait_ready_action,
            "execute": execute_action,
            "check_done": check_done_action,
            "finish": finish_action,
        }

        # 生成FSM
        outer_fsm = fsm.FSM(outer_state, outer_table)
        outer_fsm.generate(action_dict)

        return loop_counter, loop_done_reg, result

# Driver模块
class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        # 创建内层FSM
        accumulator = SimpleAccumulator()

        # 创建外层FSM
        loop_controller = OuterLoopController()
        counter, done, result = loop_controller.build(
            inner_fsm=accumulator,
            loop_start=UInt(32)(0),
            loop_end=UInt(32)(100),
            loop_step=UInt(32)(1)
        )

# 构建和运行
def test_basic_loop():
    sys = SysBuilder('basic_loop')
    with sys:
        driver = Driver()
        driver.build()

    conf = config(verilog=False, sim_threshold=1000, idle_threshold=10)
    simulator_path, _ = elaborate(sys, **conf)
    utils.run_simulator(simulator_path)
```

**预期输出**: sum = 4950

### 6.2 Multi-Cycle Computation Example

**场景**: 内层FSM执行多周期乘法操作

```python
class MultiCycleMultiplier(Module):
    """多周期乘法器（移位加法实现）"""
    def __init__(self):
        super().__init__(
            ports={
                'iteration': Port(UInt(32)),
                'valid': Port(Bits(1)),
            }
        )

    @module.combinational
    def build(self, ready_out, done_out):
        iteration, valid = self.pop_all_ports(True)

        # 状态和计算寄存器
        inner_state = RegArray(Bits(2), 1, initializer=[0])
        multiplicand = RegArray(UInt(32), 1, initializer=[0])  # 被乘数
        multiplier = RegArray(UInt(32), 1, initializer=[0])    # 乘数
        result_reg = RegArray(UInt(32), 1, initializer=[0])    # 结果
        shift_count = RegArray(UInt(8), 1, initializer=[0])    # 移位计数

        # 计算完成条件：移位32次
        compute_done = shift_count[0] >= UInt(8)(32)

        # 转移条件
        default = Bits(1)(1)
        valid_high = valid == Bits(1)(1)

        # 转移表
        inner_table = {
            "idle": {valid_high: "compute", ~valid_high: "idle"},
            "compute": {compute_done: "done", ~compute_done: "compute"},
            "done": {default: "reset"},
            "reset": {default: "idle"},
        }

        # 状态动作
        def idle_action():
            ready_out[0] = Bits(1)(1)
            done_out[0] = Bits(1)(0)
            shift_count[0] = UInt(8)(0)

        def compute_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(0)

            # 初始化被乘数和乘数
            with Condition(shift_count[0] == UInt(8)(0)):
                multiplicand[0] = iteration
                multiplier[0] = UInt(32)(3)  # 乘以3
                result_reg[0] = UInt(32)(0)

            # 移位加法算法
            with Condition(shift_count[0] < UInt(8)(32)):
                # 如果multiplier最低位为1，则加上multiplicand
                with Condition(multiplier[0][0:0] == Bits(1)(1)):
                    result_reg[0] = result_reg[0] + multiplicand[0]

                # 左移multiplicand，右移multiplier
                multiplicand[0] = multiplicand[0] << UInt(32)(1)
                multiplier[0] = multiplier[0] >> UInt(32)(1)
                shift_count[0] = shift_count[0] + UInt(8)(1)

            log("InnerFSM: Multiplying iteration={}, shift={}, partial_result={}",
                iteration, shift_count[0], result_reg[0])

        def done_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(1)
            log("InnerFSM: Multiplication done, result={}", result_reg[0])

        def reset_action():
            ready_out[0] = Bits(1)(0)
            done_out[0] = Bits(1)(0)

        action_dict = {
            "idle": idle_action,
            "compute": compute_action,
            "done": done_action,
            "reset": reset_action,
        }

        # 生成FSM
        inner_fsm = fsm.FSM(inner_state, inner_table)
        inner_fsm.generate(action_dict)

        return result_reg
```

---

## 7. Implementation Considerations

### 7.1 Parameterization

**数据位宽参数化**:
```python
class ConfigurableInnerFSM(Module):
    def __init__(self, data_width=32, counter_width=8):
        self.data_width = data_width
        self.counter_width = counter_width
        super().__init__(
            ports={
                'iteration': Port(UInt(data_width)),
                'valid': Port(Bits(1)),
            }
        )
```

**循环边界参数化**:
```python
# 外层FSM接受运行时参数
loop_controller.build(
    inner_fsm=compute_unit,
    loop_start=start_reg[0],    # 可以是寄存器值
    loop_end=end_reg[0],        # 运行时可配置
    loop_step=step_reg[0]
)
```

### 7.2 Error Handling

**超时检测**:
```python
# 在外层FSM的check_done状态添加超时计数器
timeout_counter = RegArray(UInt(16), 1, initializer=[0])

def check_done_action():
    with Condition(inner_done[0] == Bits(1)(0)):
        timeout_counter[0] = timeout_counter[0] + UInt(16)(1)
        with Condition(timeout_counter[0] > UInt(16)(1000)):
            log("ERROR: Inner FSM timeout!")
            finish()  # 或者跳转到错误处理状态
```

**握手信号验证**:
```python
# 检测非法状态（ready和done同时为高）
with Condition((inner_ready[0] == Bits(1)(1)) & (inner_done[0] == Bits(1)(1))):
    log("ERROR: Invalid handshake state!")
```

### 7.3 Performance Optimization

**流水线化**:
- 如果内层计算周期固定，可以考虑多个内层FSM并行工作
- 使用FIFO缓冲迭代数据，实现更高的吞吐量

**提前终止**:
```python
# 在内层FSM中添加提前终止条件
with Condition(result_reg[0] > threshold):
    # 立即跳转到done状态
    inner_state[0] = Bits(2)(2)  # done state
```

**周期优化**:
- 最小化状态转移开销（合并不必要的状态）
- 使用组合逻辑减少寄存器级数

---

## 8. Future Extensions

### 8.1 Multi-Level Nesting

支持三层及以上的嵌套循环：

```python
# 三层嵌套：外层 -> 中层 -> 内层
outer_loop.build(middle_loop, ...)
middle_loop.build(inner_compute, ...)
```

**挑战**:
- 握手信号传播延迟增加
- 状态机复杂度指数增长
- 调试难度提高

**解决方案**:
- 使用层次化握手协议
- 提供调试可视化工具
- 参数化嵌套层数

### 8.2 Dynamic Loop Bounds

支持运行时动态修改循环边界：

```python
# 外层FSM接受动态边界端口
class DynamicOuterLoop(Module):
    def __init__(self):
        super().__init__(
            ports={
                'new_end': Port(UInt(32)),
                'update_en': Port(Bits(1)),
            }
        )
```

**应用场景**:
- 自适应算法（根据中间结果调整迭代次数）
- 可配置硬件加速器

### 8.3 Parallel Inner FSMs

支持多个内层FSM并行处理不同迭代：

```python
# 外层FSM管理N个内层FSM
for i in range(N):
    inner_fsm[i] = InnerCompute()
    # 轮询分配迭代任务
```

**收益**:
- 提高吞吐量（N倍加速）
- 更好的资源利用率

**挑战**:
- 结果顺序保证（需要重排序逻辑）
- 资源开销增加
- 握手协议更复杂（仲裁逻辑）

---

## 9. References

- [Assassyn FSM Documentation](../../../python/assassyn/ir/module/fsm.md)
- [Assassyn Async Call Tutorial](../../../tutorials/01_async_call_en.qmd)
- [Radix Sort FSM Example](../radix_sort/main_fsm.py)

---

## Appendix A: Complete State Transition Diagrams

### Outer Loop FSM

```
     ┌──────┐
     │ init │ (Initialize loop counter)
     └───┬──┘
         │ (unconditional)
         ▼
  ┌──────────────┐
  │ wait_ready   │ (Wait for inner_ready)
  └──┬────────┬──┘
     │        │
     │ ready  │ ~ready
     │        └────┐
     ▼             │
  ┌─────────┐     │
  │ execute │     │
  └────┬────┘     │
       │          │
       │          │
       ▼          │
  ┌────────────┐  │
  │ check_done │  │
  └──┬─────┬───┘  │
     │     │      │
  done&   done&   │
  ~end    end     │
     │     │      │
     └─────┼──────┘
           │
           ▼
       ┌────────┐
       │ finish │
       └────────┘
```

### Inner Compute FSM

```
     ┌──────┐
 ┌───┤ idle │◄────┐
 │   └───┬──┘     │
 │       │ valid  │
 │ ~valid│        │
 │       ▼        │
 │   ┌─────────┐  │
 └───┤ compute │  │
     └────┬────┘  │
          │       │
    done  │       │
          ▼       │
     ┌──────┐     │
     │ done │     │
     └───┬──┘     │
         │        │
         ▼        │
     ┌───────┐    │
     │ reset │────┘
     └───────┘
```

---

**End of Specification Document**
