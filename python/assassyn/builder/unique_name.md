# Unique Name Cache

This module provicdes a cache for unique name with a given prefix.

## Exposed Class

```python
class UniqueNameCache:

    def __init__(self):
        '''
        Initialize a UniqueNameCache.
        '''
```

The constructor creates an empty cache.

----

```python
    def get_unique_name(self, prefix: str) -> str:
        '''
        Get a unique name with the given prefix.
        '''
```

For the given prefix, this method finds a unique name.
If this name does not exist before, put this name into the cache, and return itself.
If this name exists before, append a number to the prefix to make it unique, put this new name into the cache, and return the new name.