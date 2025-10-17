# CIRCT Patch

This patch disables ESI (Elastic Silicon Interfaces) to minimize dependencies.

The patch modifies `frontends/PyCDE/setup.py` to set `ESI_RUNTIME=OFF` instead of `ESI_RUNTIME=ON`, reducing the number of required dependencies for the CIRCT build.

## Patch Format

This patch uses the lightweight format:
```
3rd-party/circt/frontends/PyCDE/setup.py
-      cmake_args += ["-DESI_RUNTIME=ON"]
+      cmake_args += ["-DESI_RUNTIME=OFF"]
```

The format supports:
- `-` prefix for original lines to be replaced
- `+` prefix for replacement lines
- Exact line matching (whitespace-sensitive)
- Reverse operation support
