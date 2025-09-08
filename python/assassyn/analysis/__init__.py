"""Analysis utilities for Assassyn."""
from collections import defaultdict, deque
from ..ir.expr import Expr, FIFOPush,Bind
from .external_usage import expr_externally_used

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

    for elem in module.externals.keys():
        if isinstance(elem, Expr):
            if not isinstance(elem, FIFOPush) and not isinstance(elem, Bind):
                res.add(elem.parent.module)

    return res
