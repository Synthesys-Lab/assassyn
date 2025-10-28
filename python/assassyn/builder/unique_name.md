# Unique Name Cache

This module provides a lightweight counter-based cache for generating unique identifiers that share a common prefix. It is a core component of the Assassyn naming system, used by [TypeOrientedNamer](type_oriented_namer.md) and [NamingManager](naming_manager.md) to ensure unique, readable identifiers for IR values and modules.

## Section 1. Exposed Interfaces

### class UniqueNameCache

A cache for generating unique names with given prefixes.

#### UniqueNameCache.__init__

```python
def __init__(self) -> None
```

Initialize a UniqueNameCache.

**Explanation**: This constructor sets up an empty internal dictionary that maps each prefix to the last number that was assigned. The cache starts empty and grows as prefixes are requested through `get_unique_name`.

#### UniqueNameCache.get_unique_name

```python
def get_unique_name(self, prefix: str) -> str
```

Get a unique name with the given prefix.

**Parameters**:
- `prefix`: The prefix for the unique name

**Returns**: A unique name string. If the prefix hasn't been used, returns the prefix itself. Otherwise, appends a number to make it unique.

**Explanation**: This method implements a simple counter-based uniqueness strategy. On the first call with a given prefix, it returns the prefix unchanged and initializes the counter to 0. On subsequent calls with the same prefix, it increments the counter and returns the prefix with a numeric suffix (e.g., `foo`, `foo_1`, `foo_2`, etc.). This ensures that all returned names are unique within the cache instance while maintaining readability.

## Section 2. Internal Helpers

This module contains no internal helper functions. The `UniqueNameCache` class is self-contained with only the exposed interface methods.

## Integration with Naming System

The `UniqueNameCache` is used throughout the Assassyn naming pipeline:

- **TypeOrientedNamer**: Uses the cache to ensure unique names for IR values based on their type and operation
- **NamingManager**: Uses the cache to generate unique module names when constructing IR modules and submodules

The cache provides a simple but effective mechanism for avoiding naming conflicts while preserving semantic meaning in generated identifiers.
