# Metadata Package Entry Point

The package entry point re-exports the public surface of the Verilog metadata
implementation so callers can continue to import from
`python.assassyn.codegen.verilog.metadata` without modification.

## Exposed Interfaces

- `InteractionKind`, `InteractionMatrix`, `AsyncLedger`, `FIFOExpr` – forwarded from `core.py`.
- `ModuleBundle`, `ModuleInteractionView`, `ModuleMetadata` – forwarded from `module.py`.
- `ArrayInteractionView`, `ArrayMetadata` – forwarded from `array.py`.
- `FIFOInteractionView` – forwarded from `fifo.py`.
- `ExternalRegistry`, `ExternalRead` – forwarded from `external.py`.

Consumers should continue to import from the package root; specific submodules
are documented independently to keep responsibilities clear.
