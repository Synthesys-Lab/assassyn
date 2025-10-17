# Verilator Patch

This patch fixes GNU standard compatibility issues in Verilator by enabling C++20 support.

The patch modifies `configure.ac` to use C++20 standard instead of C++17, which resolves compatibility issues with modern GNU toolchains.

## Patch Format

This patch uses the lightweight format:
```
3rd-party/verilator/configure.ac
-_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++17)
-_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++17)
+_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++20)
+_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++20)
+#_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++17)
+#_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++17)
```

The format supports:
- `-` prefix for original lines to be replaced
- `+` prefix for replacement lines
- Multiple original lines replaced by multiple replacement lines
- Exact line matching (whitespace-sensitive)
- Reverse operation support
