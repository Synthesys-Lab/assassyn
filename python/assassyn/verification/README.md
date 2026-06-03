# Verification Package

This package contains source-level verification helpers for Assassyn-generated
simulators and RTL artifacts.

- `coverage.py`: semantic coverage JSON loading, validation, and summaries.
- `model.py`: dataclasses for normalized translation-validation models.
- `extract.py`: extraction from existing backend metadata into validation models.
- `checks.py`: static consistency checks over validation models.
- `emit.py`: JSON and monitor artifact emission helpers.
- `__init__.py`: public re-export surface.
