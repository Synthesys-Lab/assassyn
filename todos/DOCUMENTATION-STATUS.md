# Documentation Status Checklist

**Generated on**: $(date)  
**Project**: Assassyn Python Package  
**Location**: `python/assassyn/`

---

## 1. TO CHECK (31 files) - Review existing documentation against new rules

### Leaf-level files (simple to complex)
- [ ] `ir/memory/base.py` → `ir/memory/base.md` (to check)
- [ ] `ir/memory/dram.py` → `ir/memory/dram.md` (to check)
- [ ] `ir/memory/sram.py` → `ir/memory/sram.md` (to check)
- [ ] `ir/module/base.py` → `ir/module/base.md` (to check)
- [ ] `ir/module/downstream.py` → `ir/module/downstream.md` (to check)
- [ ] `ir/module/external.py` → `ir/module/external.md` (to check)
- [ ] `ir/module/fsm.py` → `ir/module/fsm.md` (to check)
- [ ] `ir/module/memorybase.py` → `ir/module/memorybase.md` (to check)
- [ ] `ir/module/module.py` → `ir/module/module.md` (to check)
- [ ] `codegen/simulator/_expr/intrinsics.py` → `codegen/simulator/_expr/intrinsics.md` (to check)
- [ ] `experimental/frontend/downstream.py` → `experimental/frontend/downstream.md` (to check)
- [ ] `experimental/frontend/factory.py` → `experimental/frontend/factory.md` (to check)
- [ ] `experimental/frontend/module.py` → `experimental/frontend/module.md` (to check)
- [ ] `ramulator2/ramulator2.py` → `ramulator2/ramulator2.md` (to check)

### Module-level documentation
- [ ] `builder.md` (to check)
- [ ] `codegen/verilog/README.md` (to check)
- [ ] `experimental/frontend/README.md` (to check)
- [ ] `experimental/README.md` (to check)
- [ ] `ir/expr/README.md` (to check)
- [ ] `ir/memory/README.md` (to check)
- [ ] `test/README.md` (to check)
- [ ] `README.md` (to check)

---

## 2. TO DOCUMENT (30 files) - Create new documentation

### Leaf-level files (simple to complex)
- [ ] `ip/multiply.py` → `ip/multiply.md` (to write)
- [ ] `codegen/simulator/node_dumper.py` → `codegen/simulator/node_dumper.md` (to write)
- [ ] `codegen/simulator/utils.py` → `codegen/simulator/utils.md` (to write)
- [ ] `codegen/verilog/cleanup.py` → `codegen/verilog/cleanup.md` (to write)
- [ ] `codegen/verilog/rval.py` → `codegen/verilog/rval.md` (to write)
- [ ] `codegen/verilog/utils.py` → `codegen/verilog/utils.md` (to write)
- [ ] `codegen/verilog/_expr/arith.py` → `codegen/verilog/_expr/arith.md` (to write)
- [ ] `codegen/verilog/_expr/array.py` → `codegen/verilog/_expr/array.md` (to write)
- [ ] `codegen/verilog/_expr/call.py` → `codegen/verilog/_expr/call.md` (to write)
- [ ] `codegen/verilog/_expr/intrinsics.py` → `codegen/verilog/_expr/intrinsics.md` (to write)
- [ ] `codegen/simulator/elaborate.py` → `codegen/simulator/elaborate.md` (to write)
- [ ] `codegen/verilog/design.py` → `codegen/verilog/design.md` (to write)
- [ ] `codegen/verilog/module.py` → `codegen/verilog/module.md` (to write)
- [ ] `codegen/verilog/system.py` → `codegen/verilog/system.md` (to write)
- [ ] `codegen/verilog/testbench.py` → `codegen/verilog/testbench.md` (to write)
- [ ] `codegen/verilog/top.py` → `codegen/verilog/top.md` (to write)
- [ ] `codegen/verilog/elaborate.py` → `codegen/verilog/elaborate.md` (to write)
- [ ] `codegen/simulator/simulator.py` → `codegen/simulator/simulator.md` (to write)
- [ ] `codegen/impl.py` → `codegen/impl.md` (to write)
- [ ] `frontend.py` → `frontend.md` (to write)
- [ ] `backend.py` → `backend.md` (to write)

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

---

## 3. DONE (16 files) - Completed documentation

### Leaf-level files (simple to complex)
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

---

## 4. Progress Summary

### Statistics
- **Total Python files**: 74
- **Files to check**: 22 (30%)
- **Files to document**: 30 (41%)
- **Files completed**: 24 (32%)

### Workflow Notes
- **Order**: Work from leaf to parent, simple to complex
- **Dependencies**: High-level summaries require lower-level documentation to be complete
- **Priority**: Focus on core functionality files first, then implementation details
- **Rules**: Each `.py` file must have a corresponding `.md` file following the new documentation standards
