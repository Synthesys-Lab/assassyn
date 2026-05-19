"""Analysis utilities for Assassyn."""
from .external_usage import (
    ExternalUsageIndex,
    build_external_usage_index,
    expr_externally_used,
)
from .timing import (
    CRITICAL_PATHS_REPORT,
    CriticalPath,
    TimingEdge,
    TimingNode,
    critical_paths,
    write_critical_paths_report,
)
from .topo import topo_downstream_modules, get_upstreams
