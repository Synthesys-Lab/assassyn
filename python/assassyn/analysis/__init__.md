# Analysis Package Interface

This package exposes reusable IR analysis helpers for Assassyn.

## Section 1. Exposed Interfaces

### External Usage Re-exports

```python
from .external_usage import (
    ExternalUsageIndex,
    build_external_usage_index,
    expr_externally_used,
)
```

These names come from [external_usage.py](./external_usage.py). They provide
precomputed and compatibility helpers for determining whether an expression
escapes its defining module.

### Timing Re-exports

```python
from .timing import (
    CriticalPath,
    TimingEdge,
    TimingNode,
    critical_paths,
    write_critical_paths_report,
)
```

These names come from [timing.py](./timing.py). They expose the pre-synthesis
critical-path analyzer and the JSON report writer used by backend
`timing_report` generation.

### Topology Re-exports

```python
from .topo import topo_downstream_modules, get_upstreams
```

These names come from [topo.py](./topo.py). They provide dependency ordering
for downstream combinational modules.

## Section 2. Internal Helpers

This package initializer intentionally defines no private helper functions. It
only re-exports analysis APIs from the concrete source modules so callers can
import from `assassyn.analysis` without depending on file layout.
