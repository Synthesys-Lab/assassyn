# Documentation Status Checklist

**Generated on**: $(date)  
**Project**: Assassyn Python Package  
**Location**: `python/assassyn/`

---

## 1. TO CHECK (24 files) - Review existing documentation against new rules

## 2. TO DOCUMENT (30 files) - Create new documentation

### Leaf-level files (simple to complex)

### Module initialization files (leaf to parent)
- [ ] `codegen/simulator/_expr/__init__.py` → `codegen/simulator/_expr/__init__.md` (to write)
- [ ] `codegen/verilog/_expr/__init__.py` → `codegen/verilog/_expr/__init__.md` (to write)
- [ ] `experimental/frontend/__init__.py` → `experimental/frontend/__init__.md` (to write)
- [ ] `ir/module/__init__.py` → `ir/module/__init__.md` (to write)
- [ ] `codegen/simulator/__init__.py` → `codegen/simulator/__init__.md` (to write)
- [ ] `codegen/verilog/__init__.py` → `codegen/verilog/__init__.md` (to write)
- [ ] `analysis/__init__.py` → `analysis/__init__.md` (to write)
- [ ] `builder/__init__.py` → `builder/__init__.md` (to write)
- [ ] `experimental/__init__.py` → `experimental/__init__.md` (to write)
- [ ] `ip/__init__.py` → `ip/__init__.md` (to write)
- [ ] `ir/__init__.py` → `ir/__init__.md` (to write)
- [ ] `ramulator2/__init__.py` → `ramulator2/__init__.md` (to write)
- [ ] `test/__init__.py` → `test/__init__.md` (to write)
- [ ] `codegen/__init__.py` → `codegen/__init__.md` (to write)
- [ ] `__init__.py` → `__init__.md` (to write)

## 2.5 Check Module Documentation When Modules Fully Documented

- [ ] `builder.md` (to check)
- [ ] `codegen/verilog/README.md` (to check)
- [ ] `experimental/frontend/README.md` (to check)
- [ ] `experimental/README.md` (to check)
- [ ] `ir/expr/README.md` (to check)
- [ ] `ir/memory/README.md` (to check)
- [ ] `test/README.md` (to check)
- [ ] `README.md` (to check)

---

## 3. DONE (23 files) - Completed documentation

- [x] `ip/multiply.py` → `ip/multiply.md` (completed)
- [x] `codegen/simulator/elaborate.py` → `codegen/simulator/elaborate.md` (completed)
- [x] `codegen/simulator/node_dumper.py` → `codegen/simulator/node_dumper.md` (completed)
- [x] `codegen/simulator/utils.py` → `codegen/simulator/utils.md` (completed)
- [x] `codegen/simulator/simulator.py` → `codegen/simulator/simulator.md` (completed)
- [x] `frontend.py` → `frontend.md` (completed)
- [x] `backend.py` → `backend.md` (completed)
- [x] `ramulator2/ramulator2.py` → `ramulator2/ramulator2.md` (completed)
- [x] `experimental/frontend/downstream.py` → `experimental/frontend/downstream.md` (completed)
- [x] `experimental/frontend/factory.py` → `experimental/frontend/factory.md` (completed)
- [x] `experimental/frontend/module.py` → `experimental/frontend/module.md` (completed)
- [x] `codegen/simulator/_expr/intrinsics.py` → `codegen/simulator/_expr/intrinsics.md` (completed)
- [x] `ir/module/base.py` → `ir/module/base.md` (completed)
- [x] `ir/module/downstream.py` → `ir/module/downstream.md` (completed)
- [x] `ir/module/external.py` → `ir/module/external.md` (completed)
- [x] `ir/module/fsm.py` → `ir/module/fsm.md` (completed)
- [x] `ir/module/module.py` → `ir/module/module.md` (completed)
- [x] `ir/expr/expr.py` → `ir/expr/expr.md` (completed)
- [x] `ir/expr/intrinsic.py` → `ir/expr/intrinsic.md` (completed)
- [x] `ir/expr/writeport.py` → `ir/expr/writeport.md` (completed)
- [x] `ir/visitor.py` → `ir/visitor.md` (completed)
- [x] `ir/expr/arith.py` → `ir/expr/arith.md` (completed)
- [x] `ir/expr/array.py` → `ir/expr/array.md` (completed)
- [x] `ir/expr/call.py` → `ir/expr/call.md` (completed)
- [x] `ir/expr/comm.py` → `ir/expr/comm.md` (completed)
- [x] `analysis/external_usage.py` → `analysis/external_usage.md` (completed)
- [x] `utils.py` → `utils.md` (completed)
- [x] `builder/naming_manager.py` → `builder/naming_manager.md` (completed)
- [x] `builder/rewrite_assign.py` → `builder/rewrite_assign.md` (completed)
- [x] `builder/type_oriented_namer.py` → `builder/type_oriented_namer.md` (completed)
- [x] `builder/unique_name.py` → `builder/unique_name.md` (completed)
- [x] `codegen/simulator/modules.py` → `codegen/simulator/modules.md` (completed)
- [x] `codegen/simulator/port_mapper.py` → `codegen/simulator/port_mapper.md` (completed)
- [x] `codegen/simulator/_expr/arith.py` → `codegen/simulator/_expr/arith.md` (completed)
- [x] `codegen/simulator/_expr/array.py` → `codegen/simulator/_expr/array.md` (completed)
- [x] `codegen/simulator/_expr/call.py` → `codegen/simulator/_expr/call.md` (completed)
- [x] `ir/array.py` → `ir/array.md` (completed)
- [x] `ir/block.py` → `ir/block.md` (completed)
- [x] `ir/const.py` → `ir/const.md` (completed)
- [x] `ir/dtype.py` → `ir/dtype.md` (completed)
- [x] `ir/value.py` → `ir/value.md` (completed)
- [x] `ir/memory/base.py` → `ir/memory/base.md` (completed)
- [x] `ir/memory/dram.py` → `ir/memory/dram.md` (completed)
- [x] `ir/memory/sram.py` → `ir/memory/sram.md` (completed)
- [x] `codegen/impl.py` → `codegen/impl.md` (completed)
- [x] `codegen/verilog/cleanup.py` → `codegen/verilog/cleanup.md` (completed)
- [x] `codegen/verilog/rval.py` → `codegen/verilog/rval.md` (completed)
- [x] `codegen/verilog/utils.py` → `codegen/verilog/utils.md` (completed)
- [x] `codegen/verilog/_expr/arith.py` → `codegen/verilog/_expr/arith.md` (completed)
- [x] `codegen/verilog/_expr/array.py` → `codegen/verilog/_expr/array.md` (completed)
- [x] `codegen/verilog/_expr/call.py` → `codegen/verilog/_expr/call.md` (completed)
- [x] `codegen/verilog/_expr/intrinsics.py` → `codegen/verilog/_expr/intrinsics.md` (completed)
- [x] `codegen/verilog/design.py` → `codegen/verilog/design.md` (completed)
- [x] `codegen/verilog/module.py` → `codegen/verilog/module.md` (completed)
- [x] `codegen/verilog/system.py` → `codegen/verilog/system.md` (completed)
- [x] `codegen/verilog/testbench.py` → `codegen/verilog/testbench.md` (completed)
- [x] `codegen/verilog/top.py` → `codegen/verilog/top.md` (completed)
- [x] `codegen/verilog/elaborate.py` → `codegen/verilog/elaborate.md` (completed)

---

## 4. Progress Summary

### Statistics
- **Total Python files**: 74
- **Files to check**: 9 (12%)
- **Files to document**: 16 (22%)
- **Files completed**: 53 (72%)

### Workflow Notes
- **Order**: Work from leaf to parent, simple to complex
- **Dependencies**: High-level summaries require lower-level documentation to be complete
- **Priority**: Focus on core functionality files first, then implementation details
- **Rules**: Each `.py` file must have a corresponding `.md` file following the new documentation standards
