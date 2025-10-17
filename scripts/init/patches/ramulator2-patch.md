# Ramulator Patch

This patch modifies Ramulator to fix local build because of:
1. The lagging C++ standard in `src/base/param.h` by adding `template` keyword right after `as<T>` on line 88.
2. In reaction to [no write transaction issue](https://github.com/CMU-SAFARI/ramulator2/issues/30), by resolving the statistics TODO for writes in `src/dram_controller/impl/generic_dram_controller.cpp`.

## Patch Format

This patch uses the lightweight format:
```
3rd-party/ramulator2/src/base/param.h
-          return _config[_name].as<T>();
+          return _config[_name].template as<T>();

3rd-party/ramulator2/src/dram_controller/impl/generic_dram_controller.cpp
-            // TODO: Add code to update statistics
+            // NEW: mirror read handling for writes
+            // Prefer a dedicated write latency if your DRAM model exposes it; else reuse read latency for now.
+            auto write_lat = m_dram->m_read_latency; // fallback for POC
+            req_it->depart = m_clk + write_lat;
+            pending.push_back(*req_it);
```

The format supports:
- `-` prefix for original lines to be replaced
- `+` prefix for replacement lines
- Multiple replacement lines for one original line
- Exact line matching (whitespace-sensitive)
- Reverse operation support
