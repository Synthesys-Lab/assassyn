"""Topological analysis utilities for Assassyn."""
from collections import defaultdict, deque
from ..ir.expr import Expr, FIFOPush, Bind
from ..ir.module.base import ModuleBase


def topo_downstream_modules(sys):
    """Topologically sort downstream modules based on their dependencies."""
    downstreams = list(sys.downstreams) if hasattr(sys, 'downstreams') else []

    graph = defaultdict(list)
    in_degree = defaultdict(int)

    for module in downstreams:
        if module not in graph:
            graph[module] = []
        if module not in in_degree:
            in_degree[module] = 0

    for module in downstreams:
        # Get upstream modules (modules this module depends on)
        upstreams = get_upstreams(module)

        # For each upstream, if it's also a downstream, add dependency
        for upstream in upstreams:
            if upstream in downstreams:
                # upstream -> module (module depends on upstream)
                graph[upstream].append(module)
                in_degree[module] += 1

    # Topological sort
    queue = deque([m for m in downstreams if in_degree[m] == 0])
    result = []

    while queue:
        module = queue.popleft()
        result.append(module)

        for neighbor in graph[module]:
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                queue.append(neighbor)

    if len(result) != len(downstreams):
        raise ValueError("Circular dependency detected in downstream modules")

    return result


def get_upstreams(module):
    """Get upstream modules of a given module.
    This matches the upstreams function in Rust.
    """
    res = set()

    externals = getattr(module, 'externals', {})
    for elem, _ in externals.items():
        if not isinstance(elem, Expr):
            continue

        if isinstance(elem, (FIFOPush, Bind)):
            continue

        parent_block = getattr(elem, 'parent', None)
        if isinstance(parent_block, ModuleBase):
            upstream_module = parent_block
        else:
            upstream_module = getattr(parent_block, 'module', None)
        if upstream_module is not None:
            res.add(upstream_module)

    return res
