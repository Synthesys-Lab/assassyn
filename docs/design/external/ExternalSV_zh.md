# ExternalSV Support Overview

## Role in the Stack
- **目标**：在 Assassyn 系统内无缝对接已有的 SystemVerilog 模块，统一仿真与 RTL 生成流程。
- **核心接口**：`ExternalSV` 继承自 `Downstream`，通过 `@external` 装饰器与 `WireIn` / `WireOut` / `RegOut` 注解描述模块边界。
- **IR 表示**：输入连接通过 `ExternalIntrinsic` 的参数完成，输出读取由 `PureIntrinsic(EXTERNAL_OUTPUT_READ)` 表示（寄存器型输出仍然通过 `RegOut[...][0]` 提醒用户注意时序）。

## 前端与 IR 细节
- `python/assassyn/ir/module/external.py` 解析注解构造 `_ExternalConfig`，并通过元类直接构造 `ExternalIntrinsic`，兼容属性访问与 `[]` 操作。
- `Singleton.peek_builder()` 提供的构建器上下文确保在缺失 `with SysBuilder` 的情况下也能生成合法的 `ExternalIntrinsic`/`PureIntrinsic` 节点，并延迟应用构造阶段的输入默认值 (`_apply_pending_connections`)。
- `ExternalIntrinsic` 与 `PureIntrinsic.EXTERNAL_OUTPUT_READ` 定义在 `python/assassyn/ir/expr/intrinsic.py`，由后端统一调度。

## Verilog 生成
- `python/assassyn/codegen/verilog/design.py:_generate_external_module_wrapper` 基于 ExternalSV 声明生成 PyCDE wrapper。
- `python/assassyn/codegen/verilog/top.py` 在 Top Harness 中区分 ExternalSV 与普通模块：延后实例化、注入跨模块 `Wire`、同步 `clk/rst`，并对 valid 信号进行双向连线。
- 每个external模块都会被一个downstream_wrapper封装接口，wrapper则在顶层Top模块下实例化被连接和使用，哪怕它在前端声明和使用的位置位于某个module或downstream内。


## 模拟器（Rust + Verilator）支持
- `python/assassyn/codegen/simulator/external.py` 收集对外可见的表达式，并生成模拟器所需的缓存与调度信息。
- `python/assassyn/codegen/simulator/verilator.py` 为每个 ExternalSV 创建独立 Verilator FFI crate，产出 `Cargo.toml`、`src/lib.rs`、`src/wrapper.cpp`、共享库和 `external_modules.json`。
- 模拟器生成的 Rust 代码会为组合模块在读出前自动调用 `eval()`，对时序模块使用 `clock_tick()` 并映射 `set_reset`/`apply_reset`。

## 典型示例
- **组合外设**：`python/ci-tests/test_easy_external.py` 展示纯组合加法器，`ext_adder.in_assign(a=a, b=b)` 直接返回 `WireOut` 结果。
- **握手同步**：`python/ci-tests/test_pipemul.py` 利用 `RegOut` 读取 `out_valid` 与乘积 `p`，结合 `Condition` 与 `async_called` 推动下游。
- **使用外部寄存器**：`python/ci-tests/test_complex_external.py` 在普通模块内部动态实例化加法器与寄存器。
