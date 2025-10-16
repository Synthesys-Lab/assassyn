<!-- 8dd25ef4-531d-4d36-9832-a3fa21f63108 3a6cb921-40f3-4a0b-aa07-a566e673e243 -->
# Unified Credit Counter Architecture Refactoring

## Overview

Replace the special-cased `trigger_counter.sv` with a unified `CreditCounter` IP module and introduce a clean activation interface at the `ModuleBase` level. This eliminates messy special cases in code generation and creates a consistent architectural pattern for Module vs Downstream activation.

**⚠️ BREAKING CHANGE**: This refactoring affects both Verilog and simulator code generators. Comprehensive testing is required at each phase.

## Core Design Principles

1. **Modules can only access external registers, not expressions** - `CreditCounter` exposes `count_reg` as `RegArray`
2. **Unified activation interface** - `ModuleBase` provides abstract activation methods implemented differently by Module (credit-based) and Downstream (dependency-based)
3. **Handle-based activation** - Use lazy handles to avoid premature AST growth
4. **Integrated wait_until** - The intrinsic creates implicit Condition blocks combining activation + user condition

## Implementation Plan

### Phase 1: Update CreditCounter IP Module

**File**: `python/assassyn/ip/credit.py`

Update the existing `CreditCounter` class to expose only registers (not expressions):

```python
class CreditCounter(Downstream):
    def __init__(self, width: int = 8, debug: bool = False):
        super().__init__()
        self.width = width
        self.debug = debug
        self.count_reg = RegArray(UInt(width), 1)
    
    @downstream.combinational
    def build(self, delta: Value, pop_ready: Value):
        count = self.count_reg[0]
        delta = delta.optional(UInt(self.width)(0))
        pop_ready = pop_ready.optional(UInt(1)(0))
        
        temp = count + delta
        pop_amount = pop_ready.select(UInt(self.width)(1), UInt(self.width)(0))
        new_count = (temp >= pop_amount).select(temp - pop_amount, UInt(self.width)(0))
        self.count_reg[0] = new_count
        return None
    
    def delta_ready(self):
        """Computed function - ir_builder handles automatically."""
        max_val = UInt(self.width)((1 << self.width) - 1)
        return self.count_reg[0] != max_val
    
    def pop_valid(self):
        """Computed function - ir_builder handles automatically."""
        return self.count_reg[0] != UInt(self.width)(0)
```

**Testing**: Run `python/ci-tests/test_ip_credit.py` to verify register-based interface works.

### Phase 2: Add Activation Handle Infrastructure

**File**: `python/assassyn/ir/module/base.py`

Add handle classes and update `ModuleBase`:

```python
class ActivationConditionHandle:
    """Lazy handle for activation condition."""
    def __init__(self, module):
        self.module = module
        self._value = None
    
    def get_value(self):
        if self._value is None:
            raise RuntimeError("Activation condition not yet computed")
        return self._value
    
    def set_value(self, value):
        self._value = value

class ModuleBase:
    def __init__(self):
        # ... existing code ...
        self._activation_condition_handle = None
        self._consume_activation_handle = None
    
    def activation_condition(self):
        """Return handle reference to activation condition."""
        if self._activation_condition_handle is None:
            self._activation_condition_handle = ActivationConditionHandle(self)
        return self._activation_condition_handle
    
    def consume_activation(self):
        """Return handle reference to activation consumption."""
        if self._consume_activation_handle is None:
            self._consume_activation_handle = ActivationConditionHandle(self)
        return self._consume_activation_handle
    
    def _compute_activation_condition(self):
        """Compute activation condition - implemented by subclasses."""
        raise NotImplementedError
    
    def _compute_consume_activation(self):
        """Compute consumption logic - implemented by subclasses."""
        raise NotImplementedError
```

**Testing**: Run `python/ci-tests/test_driver.py` to verify base infrastructure doesn't break existing code.

### Phase 3: Update Module Class with Built-in Credit Counter

**File**: `python/assassyn/ir/module/module.py`

Add credit counter to Module and implement activation interface:

```python
# Add import
from ...ip.credit import CreditCounter

class Module(ModuleBase):
    def __init__(self, ports, no_arbiter=False, credit_width=8):
        super().__init__()
        # ... existing initialization ...
        
        # Built-in credit counter
        self.credit_counter = CreditCounter(width=credit_width)
    
    def _compute_activation_condition(self):
        """Module activation: credit_counter != 0"""
        return self.credit_counter.pop_valid()
    
    def _compute_consume_activation(self):
        """Module consumption: consume credit"""
        return self.credit_counter.build(
            delta=UInt(8)(0), 
            pop_ready=UInt(1)(1)
        )
```

**Testing**: Run `make test-all` to verify no regressions.

### Phase 4: Update Downstream Class with Dependency Tracking

**File**: `python/assassyn/ir/module/downstream.py`

Add upstream dependency tracking and implement activation interface:

```python
class Downstream(ModuleBase):
    def __init__(self):
        super().__init__()
        # ... existing code ...
        self._upstream_dependencies = []
    
    def add_upstream_dependency(self, upstream_module):
        """Add upstream module dependency."""
        self._upstream_dependencies.append(upstream_module)
    
    def _compute_activation_condition(self):
        """Downstream activation: any upstream executed"""
        if not self._upstream_dependencies:
            return UInt(1)(0)
        
        conditions = [dep.activation_condition().get_value() 
                     for dep in self._upstream_dependencies]
        result = conditions[0]
        for cond in conditions[1:]:
            result = result | cond
        return result
    
    def _compute_consume_activation(self):
        """Downstream: no consumption (combinational)"""
        return None
```

**Testing**: Run `python/ci-tests/test_downstream.py` to verify downstream activation logic.

### Phase 5: Update Combinational Decorator

**File**: `python/assassyn/ir/module/base.py`

Update `combinational_for` to fill activation handles:

```python
def combinational_for(cls):
    def decorator(func):
        def wrapper(self, *args, **kwargs):
            # ... existing pre-execution setup ...
            
            result = func(self, *args, **kwargs)
            
            # Fill activation handles after build() completes
            if hasattr(self, '_activation_condition_handle') and self._activation_condition_handle:
                activation_value = self._compute_activation_condition()
                self._activation_condition_handle.set_value(activation_value)
            
            if hasattr(self, '_consume_activation_handle') and self._consume_activation_handle:
                consume_value = self._compute_consume_activation()
                self._consume_activation_handle.set_value(consume_value)
            
            # ... existing post-execution cleanup ...
            return result
        return wrapper
    return decorator
```

**Testing**: Run `make test-all` to verify decorator changes don't break IR construction.

### Phase 6: Refactor wait_until - Verilog Code Generator

**File**: `python/assassyn/codegen/verilog/_expr/intrinsics.py`

Update `wait_until` handling to use unified activation interface (around line 156):

```python
if intrinsic == Intrinsic.WAIT_UNTIL:
    cond = dumper.dump_rval(expr.args[0], False)
    
    # Get activation condition from unified interface
    module = dumper.current_module
    activation_handle = module.activation_condition()
    activation_cond = dumper.dump_rval(activation_handle.get_value(), False)
    
    # Combine activation + user condition
    combined_cond = f"({activation_cond} & {cond})"
    
    # Store for later use in cleanup
    dumper.wait_until = combined_cond
    return None
```

**Testing**: Run Verilog generation tests to verify wait_until codegen.

### Phase 7: Refactor wait_until - Simulator Code Generator

**File**: `python/assassyn/codegen/simulator/_expr/intrinsics.py`

Update `_codegen_wait_until` to use unified activation interface (around line 75):

```python
def _codegen_wait_until(node, module_ctx):
    """Generate code for wait_until intrinsic using unified activation."""
    condition = module_ctx.dump_rval(node.args[0])
    
    # Get activation condition from unified interface
    module = module_ctx.current_module
    activation_handle = module.activation_condition()
    activation_cond = module_ctx.dump_rval(activation_handle.get_value())
    
    # Combine activation + user condition
    combined_condition = f"({activation_cond} && {condition})"
    
    # Return false if condition not met (module blocks)
    return f'''
    if !({combined_condition}) {{
        return false;
    }}
    '''
```

**Testing**: Run simulator generation tests to verify wait_until works in both backends.

### Phase 8: Simplify Verilog Code Generation in cleanup.py

**File**: `python/assassyn/codegen/verilog/cleanup.py`

Replace special cases with unified activation interface (lines 101-120):

```python
def generate_execution_logic(dumper):
    """Unified execution logic for all ModuleBase types."""
    exec_conditions = []
    
    # Use unified activation interface
    activation_handle = dumper.current_module.activation_condition()
    activation_cond = dumper.dump_rval(activation_handle.get_value(), False)
    exec_conditions.append(activation_cond)
    
    # Add wait_until if present
    if dumper.wait_until:
        exec_conditions.append(f"({dumper.wait_until})")
    
    if not exec_conditions:
        dumper.append_code('executed_wire = Bits(1)(1)')
    else:
        dumper.append_code(f"executed_wire = {' & '.join(exec_conditions)}")
```

**Testing**: Run Verilog generation on all test cases to verify execution logic.

### Phase 9: Simplify Simulator Code Generation

**Primary File**: `python/assassyn/codegen/simulator/simulator.py`

The simulator backend uses an **event-based architecture** for Modules (not trigger counters like Verilog). The current implementation:

- Modules have `<module>_event` queues that track when they should execute
- Modules check `event_valid()` to see if the event timestamp is ready
- Modules are triggered when events are pushed to their queue via async calls
- Downstreams are triggered when `upstream_triggered` flags are set

**Key Insight**: The simulator backend doesn't use trigger_counter.sv at all! It uses event queues. However, we still need to update it to use the unified activation interface for consistency.

#### Changes Required:

**Lines 145-148**: Module triggered flag generation - already uses unified approach

```python
# This is already correct - triggered flag for all modules
fd.write(f"pub {module_name}_triggered : bool, ")
```

**Lines 151-153**: Event queue generation - Module-specific, keep as-is

```python
# Keep this - event queues are the simulator's credit mechanism
fd.write(f"pub {module_name}_event : VecDeque<usize>, ")
```

**Lines 254-264**: Module activation logic

```python
# CURRENT:
if not isinstance(module, Downstream):
    fd.write(f"    if self.event_valid(&self.{module_name}_event) {{\n")
else:
    upstream_conds = [f"self.{upstream_name}_triggered" for upstream in get_upstreams(module)]
    conds = " || ".join(upstream_conds) if upstream_conds else "false"
    fd.write(f"    if {conds} {{\n")

# REFACTOR TO:
# Use module.activation_condition() handle to determine activation logic
# For Module: event_valid check (credit-based via event queue)
# For Downstream: upstream triggered check (dependency-based)
```

**Lines 285**: Triggered flag update - already correct

```python
fd.write(f"      self.{module_name}_triggered = succ;\n")
```

#### Implementation Strategy:

The simulator backend activation logic is already architecturally sound:

- Modules use event queues (equivalent to credit counters)
- Downstreams use upstream dependencies
- Both set `_triggered` flags

**Minimal Changes Needed**: The simulator backend mostly needs documentation updates to clarify that:

1. Event queues ARE the credit mechanism for the simulator
2. `event_valid()` check IS the `pop_valid` equivalent
3. The unified activation interface maps cleanly to existing simulator patterns

**Optional Enhancement**: Add helper methods to make the mapping explicit:

```python
# In Module class
def _simulator_activation_check(self):
    """For simulator: check event queue validity."""
    return f"self.event_valid(&self.{namify(self.name)}_event)"

# In Downstream class  
def _simulator_activation_check(self):
    """For simulator: check upstream triggered flags."""
    upstreams = get_upstreams(self)
    conds = [f"self.{namify(u.name)}_triggered" for u in upstreams]
    return " || ".join(conds) if conds else "false"
```

**Testing**: Run simulator tests to verify module execution logic unchanged.

### Phase 9.5: Disable Verilog Backend for Isolated Simulator Testing

**Configuration Change**: Temporarily disable Verilog elaboration to independently test simulator backend correctness.

This allows us to:

1. Focus on simulator backend testing without Verilog interference
2. Verify simulator changes are correct before proceeding to Verilog refactoring
3. Isolate issues to specific backends

After this phase, run comprehensive simulator-only tests:

```bash
# Test simulator backend with Verilog disabled
python/ci-tests/test_driver.py
make test-all  # With Verilog backend disabled
```

**Note**: Re-enable Verilog backend before Phase 10.

### Phase 10: Refactor Verilog Backend to Remove trigger_counter Special Cases

**⚠️ CRITICAL ANALYSIS**: Found 24 trigger_counter references across 8 files in Verilog codegen

**⚠️ CRITICAL**: This phase affects multiple Verilog codegen files. All trigger_counter references must be removed and replaced with unified activation interface.

#### Phase 10.1: Update module.py

**File**: `python/assassyn/codegen/verilog/module.py`

Remove trigger_counter port generation (lines 47-48, 89-90, 104):

1. **Line 48**: Remove `trigger_counter_pop_valid` input port
2. **Line 89**: Remove `trigger_counter_delta_ready` input port
3. **Line 104**: Remove trigger counter output port

The activation logic will be handled through the Module's built-in credit_counter as a standard Downstream module.

#### Phase 10.2: Update top.py

**File**: `python/assassyn/codegen/verilog/top.py`

Remove manual trigger counter instantiation and wiring:

1. **Lines 97-107**: Remove TriggerCounter wire declarations
2. **Lines 227-240**: Remove TriggerCounter instantiations  
3. **Lines 551-571**: Remove trigger counter delta connections
4. **Lines 330, 438-442, 463-465**: Remove trigger_counter_pop_valid references
5. **Lines 556-560**: Remove async_callees-based trigger summing logic

The credit counter will be handled automatically through the Module's built-in credit_counter and standard downstream wiring.

#### Phase 10.3: Search for other trigger_counter references

Search the entire `python/assassyn/codegen/verilog/` directory for any remaining `trigger_counter` references:

```bash
grep -r "trigger_counter" python/assassyn/codegen/verilog/
```

Update any found references to use the unified activation interface.

**Testing**: Run full Verilog generation pipeline on all examples and test cases.

### Phase 11: Update Documentation

This phase updates all documentation to reflect the unified credit counter architecture.

#### Phase 11.1: Update IP Module Documentation

**File**: `python/assassyn/ip/credit.md`

Update API documentation to reflect register-only interface:

- Document that `count_reg` is exposed as `RegArray`
- Document that `delta_ready()` and `pop_valid()` are computed functions
- Add examples showing how Modules access the counter register
- Remove any references to returning expression tuples

#### Phase 11.2: Update Architecture Documentation

**File**: `docs/design/arch/arch.md`

Update credit-based pipeline architecture description:

- Replace `trigger_counter.sv` references with `CreditCounter` IP module
- Update diagram showing credit counter as a Downstream module
- Clarify that credit counters are built into every Module
- Document the unified activation interface for Module vs Downstream

#### Phase 11.3: Update Intrinsics Documentation

**File**: `docs/design/lang/intrinsics.md`

Update wait_until documentation (around lines 19-62):

- Document that `wait_until` combines activation condition with user condition
- Explain Module activation: `credit_counter.pop_valid() & condition`
- Explain Downstream activation: `upstream_executed & condition`
- Update examples to show the unified activation pattern

#### Phase 11.4: Update Verilog Backend Documentation

**Files**:

- `python/assassyn/codegen/verilog/README.md` (4 trigger_counter references)
- `python/assassyn/codegen/verilog/module.md` (1 reference)
- `python/assassyn/codegen/verilog/elaborate.md` (1 reference)

Remove all mentions of `trigger_counter.sv` and update to describe:

- Credit counters are now CreditCounter IP modules (Downstream)
- No more special-cased SystemVerilog files
- Activation logic uses unified interface
- Remove TriggerCounter from port generation examples
- Update resource file lists to remove trigger_counter.sv

#### Phase 11.5: Update Simulator Documentation

**File**: `docs/design/internal/simulator.md`

Update to clarify the unified activation model:

- Document that event queues ARE the credit mechanism for simulator
- Clarify that `event_valid()` is equivalent to `pop_valid`
- Document the parallel between Verilog (credit counter) and simulator (event queue)
- Update module bookkeeping section to reference unified activation interface

#### Phase 11.6: Update DONE Document

**File**: `dones/DONE-credit-counter-ip.md`

Add a note at the end documenting the migration to unified architecture:

```markdown
## Post-Implementation: Migration to Unified Architecture

After initial implementation, the CreditCounter was integrated into the unified
activation architecture (see unified-credit-counter-architecture.plan.md):
- Refactored to expose only registers (count_reg as RegArray)
- Integrated as built-in component of Module class
- Replaced trigger_counter.sv across entire Verilog backend
- Unified activation interface across Module and Downstream
```

**Testing**: Review all updated documentation for consistency and accuracy.

### Phase 12: Comprehensive Testing

Run the complete test suite to verify both backends work correctly:

```bash
# Run all tests
make test-all

# Run specific test categories
python/ci-tests/test_driver.py
python/ci-tests/test_ip_credit.py
python/ci-tests/test_downstream.py
```

Create new comprehensive test: `python/ci-tests/test_unified_activation.py`

- Test Module activation with credit counter
- Test Downstream activation with dependencies
- Test wait_until with both Module and Downstream
- Test mixed Module/Downstream systems
- Test both Verilog and simulator backends

### Phase 13: Update Existing Test Cases

Review and update test files to use new interface where needed:

- `python/ci-tests/test_ip_credit.py` - Update to use register-based interface
- `examples/*/` - Verify all examples still work
- Fix any test failures from the refactoring

## Migration Strategy

1. **Phase 1-5**: Implement infrastructure without breaking existing code
2. **Phase 6-7**: Update wait_until in BOTH backends (breaking change point)
3. **Phase 8-10**: Simplify code generation in BOTH backends
4. **Phase 11-13**: Documentation and comprehensive testing
5. **Post-migration**: Mark `trigger_counter.sv` as deprecated, remove in follow-up

## Testing Strategy

Each phase must pass tests before proceeding:

- **After Phase 1**: `test_ip_credit.py`
- **After Phase 3**: `make test-all`
- **After Phase 4**: `test_downstream.py`
- **After Phase 5**: `make test-all`
- **After Phase 6**: Verilog generation tests
- **After Phase 7**: Simulator generation tests
- **After Phase 8-9**: Full backend tests
- **After Phase 10**: All examples with Verilog backend
- **After Phase 12-13**: Complete test suite on both backends

## Risk Mitigation

1. **Incremental changes**: Each phase is testable independently
2. **Both backends**: Changes must work in both Verilog and simulator
3. **Backward compatibility**: Keep trigger_counter.sv temporarily
4. **Comprehensive testing**: Test after each major phase
5. **Rollback plan**: Can revert to trigger_counter.sv if issues arise

## Benefits

- Eliminates special cases in both `top.py` and simulator code generation
- Unified activation interface for all module types across both backends
- Clean separation: Module (credit-based) vs Downstream (dependency-based)
- Consistent with "Write Good Code" principles (no hard-coding, unified interfaces)
- Maintains testability throughout migration

### To-dos

- [ ] Update CreditCounter to expose only registers with computed functions. Test: test_ip_credit.py
- [ ] Add ActivationConditionHandle class and activation interface to ModuleBase. Test: test_driver.py
- [ ] Add built-in credit_counter to Module class and implement activation interface. Test: make test-all
- [ ] Add upstream dependency tracking to Downstream and implement activation interface. Test: test_downstream.py
- [ ] Update combinational decorator to fill activation handles after build(). Test: make test-all
- [ ] Refactor wait_until in Verilog codegen to use unified activation interface. Test: Verilog generation tests
- [ ] Refactor wait_until in simulator codegen to use unified activation interface. Test: simulator generation tests
- [ ] Simplify cleanup.py to use unified activation interface. Test: Verilog generation on all tests
- [ ] Simplify simulator modules.py to use unified activation interface. Test: simulator tests
- [ ] Remove trigger_counter special cases from top.py. Test: full Verilog pipeline on examples
- [ ] Update documentation (credit.md, arch.md, intrinsics.md)
- [ ] Create test_unified_activation.py and run complete test suite on both backends
- [ ] Update existing test cases and examples to use new interface