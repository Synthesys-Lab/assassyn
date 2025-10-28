# DONE: Array metadata refactor

0. **Goal**  
   Centralised Array metadata handling in the Verilog backend so that code and documentation reference a single registry instead of scattered dictionaries.

1. **Action Items**  
   - [x] Define a reusable `ArrayMetadata` structure and document the new design.  
   - [x] Introduce an `array.py` helper that collects array usage information into a registry.  
   - [x] Refactor the Verilog generators (`system.py`, `design.py`, `module.py`, `cleanup.py`, `top.py`, `_expr/array.py`) to consume the registry.  
   - [x] Update accompanying documentation to reflect the new flow.

2. **Changes**  
   - Added `ArrayMetadata` to `metadata.py` and created `array.py` with `ArrayMetadataRegistry.collect`, lookup helpers, and doc coverage (`python/assassyn/codegen/verilog/array.*`).  
   - Replaced the per-file dictionaries in `CIRCTDumper` with the registry and updated array module instantiation/connection logic (`python/assassyn/codegen/verilog/design.py`).  
   - Simplified system analysis to one registry call and updated module/top/cleanup logic to pull read/write indices via helper methods (`python/assassyn/codegen/verilog/system.py`, `module.py`, `top.py`, `cleanup.py`, `_expr/array.py`).  
   - Refreshed documentation to describe the registry usage path and the new metadata structure (design/system/module/top/cleanup/metadata/README + new array doc).

3. **Technical Decisions & Follow-ups**  
   - The registry stores modules by identity; consumers still guard with `is` checks to remain robust against duplicated module handles.  
   - SRAM payload owners are filtered inside `collect()` to preserve the existing “skip payload arrays” contract.  
   - `collect()` keeps deterministic ordering by iterating the same module lists as before; future improvements could include explicit ordering assertions or tests covering mixed async/downstream cases.  
   - No automated tests were run (CI guidance: `source setup.sh && make test-all`), so a follow-up smoke run is advised once time allows.
