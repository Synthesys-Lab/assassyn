# CIRCT Patch

This patch disables ESI (Elastic Silicon Interfaces) to minimize dependencies.

The patch modifies `frontends/PyCDE/setup.py` to set `ESI_RUNTIME=OFF` instead of `ESI_RUNTIME=ON`, reducing the number of required dependencies for the CIRCT build.
