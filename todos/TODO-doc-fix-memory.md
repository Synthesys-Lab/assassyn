# TODO: Memory Module Documentation Fixes

## Goal

Address documentation inconsistencies and unclear aspects discovered during the review of memory module documentation (`ir/memory/base.md`, `ir/memory/dram.md`, `ir/memory/sram.md`).

## Action Items

### Documentation Development

1. **Clarify Memory System Integration Details**
   - **Issue**: The relationship between memory modules and the broader memory system (particularly DRAM integration with ramulator2) needs clearer documentation
   - **Current State**: DRAM documentation mentions intrinsic functions but doesn't explain how these integrate with the actual memory simulation backend
   - **Required Action**: Update DRAM documentation to explain the connection between `send_read_request`/`send_write_request` intrinsics and the ramulator2 backend integration
   - **Files to Update**: `ir/memory/dram.md`, potentially `ir/expr/intrinsic.md`

2. **Document Memory Initialization File Format**
   - **Issue**: The `init_file` parameter is mentioned in all memory modules but the expected file format is not documented
   - **Current State**: Test files use `.hex` files but the format specification is unclear
   - **Required Action**: Document the expected initialization file format (hex format, byte ordering, etc.)
   - **Files to Update**: `ir/memory/base.md`, potentially create a new documentation file for memory initialization

3. **Clarify Address Width Derivation Logic**
   - **Issue**: The documentation mentions `addr_width = log2(depth)` but doesn't explain why this constraint exists
   - **Current State**: Implementation enforces power-of-2 depth but rationale is not clearly documented
   - **Required Action**: Add explanation of why power-of-2 depth is required (efficient address decoding, hardware implementation considerations)
   - **Files to Update**: `ir/memory/base.md`

### Code Development

4. **Investigate DRAM Parameter Order Inconsistency**
   - **Issue**: DRAM documentation shows `send_read_request(self, addr, re)` but implementation uses `send_read_request(self, re, addr)`
   - **Current State**: Documentation and implementation have different parameter orders
   - **Required Action**: Verify correct parameter order and update documentation to match implementation
   - **Files to Check**: `ir/memory/dram.md` vs `ir/memory/dram.py`
   - **Note**: Implementation should be considered authoritative

5. **Document Memory Module Naming Convention**
   - **Issue**: Memory modules use instance-prefixed naming (`f'{self.name}_val'`, `f'{self.name}_rdata'`) but this convention is not documented
   - **Current State**: Naming convention is used in code but not explained in documentation
   - **Required Action**: Document the naming convention for memory module internal arrays
   - **Files to Update**: `ir/memory/base.md`, `ir/memory/sram.md`

6. **Clarify SRAM Read Data Timing**
   - **Issue**: SRAM documentation mentions "last cycle `re` is enabled" but doesn't clearly explain the timing relationship
   - **Current State**: The relationship between read enable timing and `dout` buffer update is unclear
   - **Required Action**: Clarify when `dout` buffer is updated relative to read enable signal
   - **Files to Update**: `ir/memory/sram.md`

### Testing and Validation

7. **Add Memory Module Integration Tests**
   - **Issue**: While individual memory modules are tested, the integration between memory modules and the broader system could use more comprehensive testing
   - **Current State**: Basic functionality tests exist but edge cases and integration scenarios are not fully covered
   - **Required Action**: Review existing tests and add integration test cases for memory module interactions
   - **Files to Create/Update**: `ci-tests/test_memory_integration.py`

8. **Document Memory Module Performance Characteristics**
   - **Issue**: No documentation exists about the performance characteristics or limitations of memory modules
   - **Current State**: Implementation details are documented but performance implications are not
   - **Required Action**: Add performance notes to memory module documentation (latency, throughput, resource usage)
   - **Files to Update**: `ir/memory/base.md`, `ir/memory/sram.md`, `ir/memory/dram.md`

## Notes

- All identified issues are documentation-related and do not affect the correctness of the implementation
- The memory module implementations appear to be functionally correct based on test analysis
- Priority should be given to clarifying the DRAM parameter order inconsistency (#4) as this could lead to confusion for users
- The memory initialization file format documentation (#2) would be particularly valuable for users trying to initialize memory modules
