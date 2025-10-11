# Documentation Status Checklist

**Generated on**: $(date)  
**Project**: Assassyn Python Package  
**Location**: `python/assassyn/`

---

## 📋 TO CHECK (Existing Documentation)

*Status: Need to verify compliance with new documentation rules*

### Root Level
- [ ] `utils.py` → `utils.md` (to check)
- [ ] `README.md` (folder documentation - to check)

### Analysis Module
- [ ] `analysis/external_usage.py` → `analysis/external_usage.md` (to check)
- [ ] `analysis/topo.py` → `analysis/topo.md` (to check)

### Builder Module
- [ ] `builder/naming_manager.py` → `builder/naming_manager.md` (to check)
- [ ] `builder/rewrite_assign.py` → `builder/rewrite_assign.md` (to check)
- [ ] `builder/type_oriented_namer.py` → `builder/type_oriented_namer.md` (to check)
- [ ] `builder/unique_name.py` → `builder/unique_name.md` (to check)
- [ ] `builder.md` (module documentation - to check)

### Codegen/Simulator Module
- [ ] `codegen/simulator/modules.py` → `codegen/simulator/modules.md` (to check)
- [ ] `codegen/simulator/port_mapper.py` → `codegen/simulator/port_mapper.md` (to check)
- [ ] `codegen/simulator/_expr/arith.py` → `codegen/simulator/_expr/arith.md` (to check)
- [ ] `codegen/simulator/_expr/array.py` → `codegen/simulator/_expr/array.md` (to check)
- [ ] `codegen/simulator/_expr/call.py` → `codegen/simulator/_expr/call.md` (to check)
- [ ] `codegen/simulator/_expr/intrinsics.py` → `codegen/simulator/_expr/intrinsics.md` (to check)

### Codegen/Verilog Module
- [ ] `codegen/verilog/README.md` (folder documentation - to check)

### Experimental Module
- [ ] `experimental/frontend/downstream.py` → `experimental/frontend/downstream.md` (to check)
- [ ] `experimental/frontend/factory.py` → `experimental/frontend/factory.md` (to check)
- [ ] `experimental/frontend/module.py` → `experimental/frontend/module.md` (to check)
- [ ] `experimental/frontend/README.md` (folder documentation - to check)
- [ ] `experimental/README.md` (folder documentation - to check)

### IR Module
- [ ] `ir/array.py` → `ir/array.md` (to check)
- [ ] `ir/block.py` → `ir/block.md` (to check)
- [ ] `ir/const.py` → `ir/const.md` (to check)
- [ ] `ir/dtype.py` → `ir/dtype.md` (to check)
- [ ] `ir/value.py` → `ir/value.md` (to check)
- [ ] `ir/visitor.py` → `ir/visitor.md` (to check)

### IR/Expr Module
- [ ] `ir/expr/arith.py` → `ir/expr/arith.md` (to check)
- [ ] `ir/expr/array.py` → `ir/expr/array.md` (to check)
- [ ] `ir/expr/call.py` → `ir/expr/call.md` (to check)
- [ ] `ir/expr/comm.py` → `ir/expr/comm.md` (to check)
- [ ] `ir/expr/expr.py` → `ir/expr/expr.md` (to check)
- [ ] `ir/expr/intrinsic.py` → `ir/expr/intrinsic.md` (to check)
- [ ] `ir/expr/writeport.py` → `ir/expr/writeport.md` (to check)
- [ ] `ir/expr/README.md` (folder documentation - to check)

### IR/Memory Module
- [ ] `ir/memory/base.py` → `ir/memory/base.md` (to check)
- [ ] `ir/memory/dram.py` → `ir/memory/dram.md` (to check)
- [ ] `ir/memory/sram.py` → `ir/memory/sram.md` (to check)
- [ ] `ir/memory/README.md` (folder documentation - to check)

### IR/Module Module
- [ ] `ir/module/base.py` → `ir/module/base.md` (to check)
- [ ] `ir/module/downstream.py` → `ir/module/downstream.md` (to check)
- [ ] `ir/module/external.py` → `ir/module/external.md` (to check)
- [ ] `ir/module/fsm.py` → `ir/module/fsm.md` (to check)
- [ ] `ir/module/memorybase.py` → `ir/module/memorybase.md` (to check)
- [ ] `ir/module/module.py` → `ir/module/module.md` (to check)

### Ramulator2 Module
- [ ] `ramulator2/ramulator2.py` → `ramulator2/ramulator2.md` (to check)

### Test Module
- [ ] `test/README.md` (folder documentation - to check)

---

## ✍️ TO DOCUMENT (Missing Documentation)

*Status: Need to create documentation following new rules*

### Root Level
- [ ] `__init__.py` → `__init__.md` (to write)
- [ ] `frontend.py` → `frontend.md` (to write)
- [ ] `backend.py` → `backend.md` (to write)

### Analysis Module
- [ ] `analysis/__init__.py` → `analysis/__init__.md` (to write)

### Builder Module
- [ ] `builder/__init__.py` → `builder/__init__.md` (to write)

### Codegen Module
- [ ] `codegen/__init__.py` → `codegen/__init__.md` (to write)
- [ ] `codegen/impl.py` → `codegen/impl.md` (to write)

### Codegen/Simulator Module
- [ ] `codegen/simulator/__init__.py` → `codegen/simulator/__init__.md` (to write)
- [ ] `codegen/simulator/elaborate.py` → `codegen/simulator/elaborate.md` (to write)
- [ ] `codegen/simulator/node_dumper.py` → `codegen/simulator/node_dumper.md` (to write)
- [ ] `codegen/simulator/simulator.py` → `codegen/simulator/simulator.md` (to write)
- [ ] `codegen/simulator/utils.py` → `codegen/simulator/utils.md` (to write)
- [ ] `codegen/simulator/_expr/__init__.py` → `codegen/simulator/_expr/__init__.md` (to write)

### Codegen/Verilog Module
- [ ] `codegen/verilog/__init__.py` → `codegen/verilog/__init__.md` (to write)
- [ ] `codegen/verilog/cleanup.py` → `codegen/verilog/cleanup.md` (to write)
- [ ] `codegen/verilog/design.py` → `codegen/verilog/design.md` (to write)
- [ ] `codegen/verilog/elaborate.py` → `codegen/verilog/elaborate.md` (to write)
- [ ] `codegen/verilog/module.py` → `codegen/verilog/module.md` (to write)
- [ ] `codegen/verilog/rval.py` → `codegen/verilog/rval.md` (to write)
- [ ] `codegen/verilog/system.py` → `codegen/verilog/system.md` (to write)
- [ ] `codegen/verilog/testbench.py` → `codegen/verilog/testbench.md` (to write)
- [ ] `codegen/verilog/top.py` → `codegen/verilog/top.md` (to write)
- [ ] `codegen/verilog/utils.py` → `codegen/verilog/utils.md` (to write)
- [ ] `codegen/verilog/_expr/__init__.py` → `codegen/verilog/_expr/__init__.md` (to write)
- [ ] `codegen/verilog/_expr/arith.py` → `codegen/verilog/_expr/arith.md` (to write)
- [ ] `codegen/verilog/_expr/array.py` → `codegen/verilog/_expr/array.md` (to write)
- [ ] `codegen/verilog/_expr/call.py` → `codegen/verilog/_expr/call.md` (to write)
- [ ] `codegen/verilog/_expr/intrinsics.py` → `codegen/verilog/_expr/intrinsics.md` (to write)

### Experimental Module
- [ ] `experimental/__init__.py` → `experimental/__init__.md` (to write)
- [ ] `experimental/frontend/__init__.py` → `experimental/frontend/__init__.md` (to write)

### IP Module
- [ ] `ip/multiply.py` → `ip/multiply.md` (to write)

### IR Module
- [ ] `ir/__init__.py` → `ir/__init__.md` (to write)
- [ ] `ir/module/__init__.py` → `ir/module/__init__.md` (to write)

### Ramulator2 Module
- [ ] `ramulator2/__init__.py` → `ramulator2/__init__.md` (to write)

### Test Module
- [ ] `test/__init__.py` → `test/__init__.md` (to write)

---

## ✅ DONE (Completed Documentation)

*Status: Documentation completed and verified*

*No items completed yet.*

---

## 📊 Progress Summary

- **Total Python files**: 75
- **Files to check**: 45 (60%)
- **Files to document**: 30 (40%)
- **Files completed**: 0 (0%)
- **Overall progress**: 0%
