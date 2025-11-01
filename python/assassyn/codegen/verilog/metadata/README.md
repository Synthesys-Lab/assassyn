# Metadata Package

This package splits the Verilog metadata implementation into focused modules:

- `core.py` – shared enums, `InteractionMatrix`, and the async ledger.
- `module.py` – module-scoped helpers (`ModuleBundle`, `ModuleInteractionView`, `ModuleMetadata`).
- `array.py` – array projections and `ArrayMetadata`.
- `fifo.py` – FIFO projections.
- `external.py` – external module registry tracking classes, instances, and cross-module reads.
- `__init__.py` – public re-export surface, keeping the legacy import path intact.

See `../metadata.md` for a high-level overview of how these modules collaborate during code generation.
