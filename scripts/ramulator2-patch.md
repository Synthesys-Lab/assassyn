# Ramulator Patch

This patch modifies Ramulator to fix local build because of:
1. The lagging C++ standard in `src/base/param.h` by adding `template` keyword right after `as<T>` on line 88.
2. Add `.dylib` to `.gitignore`.
3. In reaction to [no write transaction issue](https://github.com/CMU-SAFARI/ramulator2/issues/30), by resolving the statistics TODO for writes in `src/dram_controller/impl/generic_dram_controller.cpp`.
