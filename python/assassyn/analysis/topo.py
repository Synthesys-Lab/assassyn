"""Topological sorting utilities for module dependency analysis."""

from collections import defaultdict, deque


def topological_sort(modules, deps):
    """
    Perform topological sort on modules based on their dependencies.

    Args:
        modules: List of modules to sort
        deps: Dictionary mapping each module to its set of dependencies

    Returns:
        List of modules in topological order (dependencies first)
    """
    # Calculate in-degree (number of modules that depend on this module)
    dependents = defaultdict(set)
    for module, dependencies in deps.items():
        for dep in dependencies:
            dependents[dep].add(module)

    in_degree = {m: len(deps.get(m, set())) for m in modules}

    # Start with modules that have no dependencies
    queue = deque([m for m in modules if in_degree[m] == 0])
    sorted_modules = []

    while queue:
        module = queue.popleft()
        sorted_modules.append(module)

        # For each module that depends on this one
        for dependent in dependents[module]:
            in_degree[dependent] -= 1
            if in_degree[dependent] == 0:
                queue.append(dependent)

    # Handle any cycles by adding remaining modules
    for m in modules:
        if m not in sorted_modules:
            sorted_modules.append(m)

    return sorted_modules
