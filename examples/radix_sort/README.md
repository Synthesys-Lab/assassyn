# Radix Sort 示例

这个目录包含使用 Assassyn 实现的硬件 radix sort 算法。

## 文件说明

- **`main_fsm.py`**: 使用 FSM 模块的重构版本（推荐）
  - 清晰的声明式状态机定义
  - 易于理解和维护
  - 完整的文档和注释

- **`main.py`**: 原始实现（使用手动状态管理）
  - 注意：此文件包含过时的 SRAM API 调用，需要修复

- **`radix_sort.py`**: Python 参考实现
  - 用于理解算法逻辑
  - 不是硬件实现

- **`test_radix_sort.py`**: 测试框架
  - 用于比较不同实现的输出

- **`workload/numbers.data`**: 测试数据
  - 包含 2048 个 32 位十六进制数

## 算法说明

### Radix Sort 基础

Radix sort 是一种非比较排序算法，通过处理数字的每一位来排序。本实现使用：

- **Radix-16**: 每次处理 4 位（0-15）
- **8 次遍历**: 32 位 ÷ 4 位 = 8 次
- **稳定排序**: 保持相同键值元素的相对顺序

### 硬件实现特点

1. **Ping-pong 缓冲**:
   - 内存分为两半
   - 每次遍历从一半读取，写入另一半
   - 避免覆盖源数据

2. **流水线阶段**:
   - **Stage 1 (Read)**: 读取数据，构建直方图
   - **Stage 2 (Prefix)**: 计算前缀和（桶边界）
   - **Stage 3 (Write)**: 根据桶位置写回排序数据

3. **两层 FSM**:
   - **主 FSM**: 控制整体流程（reset → read → prefix → write）
   - **MemImpl FSM**: 处理写回阶段（init → read → write → reset）

## 运行示例

```bash
# 设置环境
source ../../setup.sh

# 运行 FSM 版本（推荐）
cd examples/radix_sort
python3 main_fsm.py

# 查看参考实现
python3 radix_sort.py
```

## FSM 实现详解

### 主 FSM 状态（Driver 模块）

```
reset (0) → read (1) → prefix (2) → write (3) → reset
   ↑                                               |
   └───────────────────────────────────────────────┘
```

#### 状态说明

1. **reset**:
   - 初始化下一轮排序
   - 增加位偏移（0→4→8→...→28）
   - 切换 ping-pong 缓冲区
   - 设置内存读取

2. **read**:
   - 从当前缓冲区读取所有元素
   - 提取当前位偏移的 4 位基数
   - 通过 MemUser 增加桶计数器
   - 读完所有元素后转换

3. **prefix**:
   - 将桶计数转换为位置（前缀和）
   - 需要 16 个周期（每个桶一个）
   - 前缀和完成后转换

4. **write**:
   - 委托给 MemImpl FSM
   - MemImpl 读取、排序并写入元素
   - 返回 reset 进行下一轮

### MemImpl FSM 状态（写回阶段）

```
init (0) → read (1) → write (2) → read/reset
                        ↓
                    reset (3) → init
```

#### 状态说明

1. **init**: 初始化读/写地址指针
2. **read**: 设置从内存读取下一个元素
3. **write**:
   - 写入数据到目标位置
   - 更新基数计数器
   - 如果未完成，返回 read
   - 如果完成，转到 reset
4. **reset**:
   - 清除所有基数计数器
   - 为下一轮准备
   - 返回主 FSM 的 reset 状态

### 关键数据结构

- **radix_reg[16]**: 基数直方图/前缀和数组
  - 读取阶段：存储每个桶的计数
  - 前缀和阶段：转换为桶边界位置
  - 写入阶段：用作递减计数器

- **offset_reg**: 当前处理的位偏移（0, 4, 8, ..., 28）

- **mem_pingpong_reg**: 缓冲区选择器（0 或 1）

- **SM_reg**: 主状态机寄存器（2 位，4 个状态）

- **SM_MemImpl**: MemImpl 状态机寄存器（2 位，4 个状态）

## FSM 模块使用

本实现展示了如何使用 Assassyn 的 FSM 模块：

```python
# 定义转换表
transition_table = {
    "state_name": {condition1: "next_state1", condition2: "next_state2"},
    ...
}

# 创建 FSM 实例
my_fsm = fsm.FSM(state_reg, transition_table)

# 定义状态特定的动作
action_dict = {
    "state_name": action_function,
    ...
}

# 生成 FSM 逻辑
my_fsm.generate(action_dict)
```

### FSM 的优势

1. **声明式**: 状态转换在表中明确定义
2. **可读性**: 易于理解状态机的结构
3. **可维护性**: 修改状态或转换很简单
4. **一致性**: 与其他 Assassyn 示例保持一致

## 技术要点

### SRAM 连接

```python
# 正确的方式（当前 API）
numbers_mem.build(we[0], re[0], addr_reg[0], wdata[0])
memory_user.async_called(rdata=numbers_mem.dout[0])
```

SRAM.dout 是包含读取数据的 RegArray，通过 async_called 连接到 MemUser 的 rdata 端口。

### 类型转换

Assassyn 严格区分 Bits 和 UInt/Int 类型：

```python
# ~ 运算符返回 Bits 类型
# 需要 bitcast 转换为 UInt
mem_pingpong_reg[0] = (~mem_pingpong_reg[0]).bitcast(UInt(1))
```

### Ping-pong 缓冲机制

```
Pass 1: Read [0...N)    → Write [N...2N)
Pass 2: Read [N...2N)   → Write [0...N)
Pass 3: Read [0...N)    → Write [N...2N)
...
```

每次遍历切换源和目标，使用 `mem_pingpong_reg` 控制。

## 性能特性

- **吞吐量**: 每个周期处理一个元素（读取阶段）
- **延迟**: O(k×n)，其中 k=8（遍历次数），n=元素数量
- **内存**: 需要 2×n 空间用于 ping-pong 缓冲
- **并行性**: 可以流水线化多个排序操作

## 扩展和改进

### 可能的优化

1. **并行桶计数**: 使用多个计数器并行处理
2. **更大的基数**: 使用 8 位基数减少遍历次数（但增加桶数量）
3. **混合排序**: 对小数据集切换到插入排序
4. **多路合并**: 一次处理多个元素

### 学习资源

- FSM 模块文档: `python/assassyn/ir/module/fsm.md`
- SRAM 模块文档: `python/assassyn/ir/memory/sram.md`
- 另一个 FSM 示例: `examples/spmv/spmv_fsm.py`
- 设计文档: `docs/design/`

## 故障排除

### 常见问题

1. **SRAM API 错误**: 确保只传递 4 个参数给 SRAM.build()
2. **类型不匹配**: 使用 .bitcast() 在 Bits 和 UInt/Int 之间转换
3. **状态编码**: FSM 使用 floor(log2(num_states)) 位，确保 state_reg 足够大

### 调试技巧

- 使用 log() 语句跟踪状态转换
- 检查 radix_reg 值以验证直方图和前缀和
- 比较每次遍历后的内存内容
- 验证 ping-pong 切换正确

## 许可证

本示例是 Assassyn 项目的一部分。
