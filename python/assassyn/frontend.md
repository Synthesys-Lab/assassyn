# Frontend Module

This module provides the main programming interface for Assassyn, exposing all the core IR components, builders, and expressions that users need to construct hardware designs. It serves as the primary entry point for hardware description in Assassyn.

---

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures in this source file unit that are exposed to the usage for other parts of the project.

### Module Imports

This module primarily serves as a re-export interface, making core Assassyn components available through a single import. All imports are marked with `#pylint: disable=unused-import` since this module's purpose is to expose these components to external users.

**Exposed Components:**

#### Array Types
- `RegArray`: Register array implementation for hardware registers
- `Array`: Generic array data structure for hardware arrays

#### Data Types
- `DType`: Base data type interface
- `Int`: Signed integer data type
- `UInt`: Unsigned integer data type  
- `Float`: Floating-point data type
- `Bits`: Bit-vector data type
- `Record`: Record/struct data type

#### Builder System
- `SysBuilder`: Main system builder for constructing hardware systems
- `ir_builder`: IR builder context manager
- `Singleton`: Singleton pattern implementation for unique naming
- `rewrite_assign`: Assignment rewriting functionality

#### Expression System
- `Expr`: Base expression interface
- `log`: Logging expression for debugging
- `concat`: Concatenation expression
- `finish`: Finish/termination expression
- `wait_until`: Wait condition expression
- `assume`: Assumption expression for verification
- `barrier`: Synchronization barrier expression
- `send_read_request`: Memory read request expression
- `send_write_request`: Memory write request expression
- `has_mem_resp`: Memory response check expression

#### Module System
- `Module`: Base module interface
- `Port`: Port interface for module communication
- `Downstream`: Downstream module for combinational logic
- `fsm`: Finite state machine module

#### Memory Systems
- `SRAM`: Static RAM memory implementation
- `DRAM`: Dynamic RAM memory implementation

#### Control Flow
- `Condition`: Conditional execution block
- `Cycle`: Cycle-based execution block

#### Value System
- `Value`: Base value interface

#### Module Utilities
- `module`: Module utility functions
- `downstream`: Downstream module utilities

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. It serves purely as a re-export interface to expose core Assassyn functionality through a single import point.

---

## Usage Pattern

The frontend module is typically imported as:

```python
import assassyn.frontend as assassyn
# or
from assassyn import frontend
```

This allows users to access all core Assassyn functionality through a single import, providing a clean and unified API for hardware design. The module follows the design principle of exposing all necessary components while maintaining a simple and consistent interface.

The exposed components support the credit-based pipeline architecture described in the [pipeline design document](../../docs/design/internal/pipeline.md) and the module generation system described in the [module design document](../../docs/design/internal/module.md).
