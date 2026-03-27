# 流水线化写入设计文档

## 问题分析

### 当前瓶颈

根据性能基线分析，**写入阶段占66%的总运行时间**，这是主要瓶颈。

**当前写入流程** (每个元素需要2周期):
```
Cycle N:   读取element[i]的准备 (设置read地址)
Cycle N+1: 写入element[i] (数据可用，计算write地址并写入)
Cycle N+2: 读取element[i+1]的准备
Cycle N+3: 写入element[i+1]
...
总计: 2048个元素 × 2周期 = 4096周期/遍历
```

**根本原因**:
1. 单端口SRAM：不能同时读和写
2. SRAM读取延迟：数据在下一周期才可用
3. 地址依赖：写地址依赖于读取的数据（radix值）

## 解决方案：双SRAM流水线架构

### 核心思想

使用**两个独立的SRAM**，一个专门用于读取，另一个专门用于写入。这样可以在同一周期内：
- 从read_sram读取element[i+1]
- 向write_sram写入element[i]

### 流水线时序

```
Cycle N:   读取element[0] (预取)
Cycle N+1: 读取element[1], 写入element[0] (流水线开始)
Cycle N+2: 读取element[2], 写入element[1]
...
Cycle N+2048: 读取完成, 写入element[2047]
总计: 1 + 2048 = 2049周期/遍历 (vs 原来的4096)
```

**加速比**: 4096 / 2049 ≈ **2.0×**

### 架构设计

```
           Pass 1              Pass 2              Pass 3
         ┌─────────┐         ┌─────────┐         ┌─────────┐
Read  ─> │ SRAM_A  │ ──read─>│ SRAM_B  │ ──read─>│ SRAM_A  │
         └─────────┘         └─────────┘         └─────────┘
              │                   │                   │
            write               write               write
              ↓                   ↓                   ↓
         ┌─────────┐         ┌─────────┐         ┌─────────┐
Write ─> │ SRAM_B  │         │ SRAM_A  │         │ SRAM_B  │
         └─────────┘         └─────────┘         └─────────┘

Ping-pong: 每次遍历后交换读写角色
```

### 关键设计点

1. **双SRAM实例**
   - `sram_a`: 初始时包含输入数据
   - `sram_b`: 初始时为空

2. **Ping-pong控制**
   - 每次遍历后交换读写角色
   - 使用`mem_pingpong_reg`控制选择

3. **流水线状态机**
   ```
   MemImpl FSM (新):
   - init:     初始化，预取第一个元素
   - pipeline: 同时读取i+1和写入i (主循环)
   - drain:    写入最后一个元素
   - reset:    清零radix_reg，返回主FSM
   ```

4. **数据缓冲**
   - 需要一个寄存器暂存读取的数据
   - `rdata_buffer`: 存储上一周期读取的数据

## 实现细节

### 1. SRAM创建和连接

```python
# 创建两个SRAM实例
sram_a = SRAM(
    width=data_width,
    depth=2**addr_width,
    init_file=f"{resource_base}/numbers.data"
)
sram_a.name = "sram_a"

sram_b = SRAM(
    width=data_width,
    depth=2**addr_width
)
sram_b.name = "sram_b"

# 根据ping-pong选择读写SRAM
with Condition(mem_pingpong_reg[0] == UInt(1)(0)):
    read_sram = sram_a
    write_sram = sram_b
with Condition(mem_pingpong_reg[0] == UInt(1)(1)):
    read_sram = sram_b
    write_sram = sram_a
```

**注意**: Assassyn的条件赋值可能不支持这种动态选择。需要改用控制信号选择。

### 2. MemImpl流水线FSM

```python
class MemImpl(Downstream):
    def build(self, ...):
        # 状态：0=init, 1=pipeline, 2=drain, 3=reset
        SM_MemImpl = RegArray(UInt(2), 1, initializer=[0])

        # 地址寄存器
        read_addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        write_addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])

        # 数据缓冲
        rdata_buffer = RegArray(Bits(data_width), 1, initializer=[0])

        with Condition(SM_reg[0] == UInt(2)(3)):  # Stage 3
            # init: 预取第一个元素
            with Condition(SM_MemImpl[0] == UInt(2)(0)):
                # 设置read_sram读取第一个元素
                # 初始化地址
                SM_MemImpl[0] = UInt(2)(1)

            # pipeline: 主循环
            with Condition(SM_MemImpl[0] == UInt(2)(1)):
                # 同时：
                # 1. 写入缓冲的数据到write_sram
                # 2. 从read_sram读取下一个元素到缓冲
                # 3. 检查是否完成

                with Condition(read_addr_reg[0] > mem_start):
                    # 继续流水线
                    SM_MemImpl[0] = UInt(2)(1)
                with Condition(read_addr_reg[0] == mem_start):
                    # 进入drain
                    SM_MemImpl[0] = UInt(2)(2)

            # drain: 写入最后一个元素
            with Condition(SM_MemImpl[0] == UInt(2)(2)):
                # 写入缓冲的最后一个元素
                SM_MemImpl[0] = UInt(2)(3)

            # reset: 清零
            with Condition(SM_MemImpl[0] == UInt(2)(3)):
                # 重置radix_reg
                # 返回主FSM
```

### 3. 读写控制

由于不能动态选择SRAM，需要为每个SRAM单独设置控制信号：

```python
# SRAM A控制
sram_a_we = RegArray(Bits(1), 1, initializer=[0])
sram_a_re = RegArray(Bits(1), 1, initializer=[0])
sram_a_addr = RegArray(UInt(addr_width), 1, initializer=[0])
sram_a_wdata = RegArray(Bits(data_width), 1, initializer=[0])

# SRAM B控制
sram_b_we = RegArray(Bits(1), 1, initializer=[0])
sram_b_re = RegArray(Bits(1), 1, initializer=[0])
sram_b_addr = RegArray(UInt(addr_width), 1, initializer=[0])
sram_b_wdata = RegArray(Bits(data_width), 1, initializer=[0])

# 根据ping-pong设置控制信号
with Condition(mem_pingpong_reg[0] == UInt(1)(0)):
    # A读，B写
    sram_a_re[0] = Bits(1)(1)
    sram_a_we[0] = Bits(1)(0)
    sram_a_addr[0] = read_addr_reg[0]

    sram_b_re[0] = Bits(1)(0)
    sram_b_we[0] = Bits(1)(1)
    sram_b_addr[0] = write_addr_reg[0]
    sram_b_wdata[0] = rdata_buffer[0]
```

## 挑战和注意事项

### 1. Assassyn的限制

**问题**: Assassyn不支持在`Condition`内动态选择对象

```python
# ❌ 不支持
with Condition(x == 0):
    sram = sram_a
with Condition(x == 1):
    sram = sram_b
sram.build(...)  # sram引用不明确
```

**解决方案**: 为每个SRAM单独生成控制逻辑

### 2. 数据缓冲时序

需要仔细管理数据缓冲：
- 第N周期：读取element[i]，数据在第N+1周期可用
- 第N+1周期：使用缓冲的element[i-1]写入，同时接收element[i]

### 3. 初始预取

流水线需要一个周期的预热：
- init状态：读取第一个元素
- pipeline状态：开始重叠读写

### 4. 边界条件

- 第一个元素：只读不写（init）
- 最后一个元素：只写不读（drain）

## 预期性能

### 周期数估算

**每次遍历**:
```
Reset:      1周期
Read:       2048周期 (不变)
Prefix:     16周期 (不变)
Write:
  - Init:   1周期 (预取)
  - Pipeline: 2047周期 (2048-1)
  - Drain:  1周期
  - Reset:  17周期
  Total Write: 2066周期 (vs 原来4113)

每遍历总计: 1 + 2048 + 16 + 2066 = 4131周期 (vs 原来6178)
```

**8次遍历**: 4131 × 8 = **33,048周期**

**对比**:
- 原始: 49,441周期
- 流水线: 33,048周期
- **加速**: 1.50× (33%性能提升)

### 资源成本

**内存**:
- 原始: 1个SRAM × 16KB = 16KB
- 流水线: 2个SRAM × 16KB = **32KB** (2×)

**寄存器**: 不变 (~640 bits)

**权衡**: 用2×内存换1.5×性能

## 实现计划

1. **从main.py复制创建main_pipelined.py**
2. **修改Driver.build()添加双SRAM**
3. **重写MemImpl.build()实现流水线FSM**
4. **测试和调试**
5. **运行benchmark验证性能**

## 参考

- 当前实现: `examples/radix_sort/main.py`
- 性能基线: `examples/radix_sort/baseline_main.txt`
- 原始TODO: `TODO-radix-sort-optimization.md`
