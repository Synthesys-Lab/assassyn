# TODO: Documentation Fix for Array Code Generation

## Goal

Fix documentation inconsistencies and improve clarity for the array code generation module (`codegen/simulator/_expr/array.py`) to align with the new documentation standards and provide accurate technical details.

## Action Items

### Document Development

- [x] **Update array.md documentation structure** - Reorganized the documentation to follow the new standard structure with proper sections for Summary, Exposed Interfaces, and detailed function documentation with parameter descriptions and return value explanations.

- [x] **Fix generated code examples** - Corrected the `codegen_array_write` documentation to show the actual generated code using the `write()` method with port index instead of the incorrect `write_port.push()` method.

- [x] **Add project-specific context** - Enhanced the documentation with references to the simulator architecture, half-cycle timing mechanism, and port-based write management system.

### Issues Identified and Resolved

1. **Incorrect Generated Code Example**: The original documentation showed `sim.<array_name>.write_port.push(ArrayWrite::new(...))` but the actual implementation uses `sim.<array_name>.write(<port_idx>, write)`. This has been corrected.

2. **Missing Function Documentation**: The functions lacked proper docstrings with parameter descriptions and return value explanations. Added comprehensive docstrings following the new documentation standards.

3. **Lack of Context**: The documentation didn't explain the relationship to the simulator architecture, timing model, or port management system. Added references to relevant design documents and explained the half-cycle timing mechanism.

4. **Documentation Structure**: The original documentation didn't follow the new standard structure. Reorganized into proper sections with detailed explanations.

### Technical Details Clarified

- **Half-cycle timing**: The timestamp calculation `sim.stamp - sim.stamp % 100 + 50` aligns writes to the half-cycle boundary for proper register update timing.

- **Port-based writes**: The system uses compile-time port indices assigned by the port manager for optimal performance with multi-writer arrays.

- **Deferred writes**: Array writes are buffered and applied during the next half-cycle when `tick_registers()` is called, following the simulator's timing model.

### No Code Changes Required

The implementation in `array.py` is correct and consistent with the documentation. No functional changes are needed as the code properly implements the array read/write operations according to the simulator architecture.

## Status

✅ **Completed** - Documentation has been updated and moved to the DONE section in DOCUMENTATION-STATUS.md. The documentation now accurately reflects the implementation and provides comprehensive context for understanding the array code generation functions.
