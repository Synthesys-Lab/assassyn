"""Analysis utilities for Assassyn."""
from .external_usage import (
    ExternalUsageIndex,
    build_external_usage_index,
    expr_externally_used,
)
from .topo import topo_downstream_modules, get_upstreams
