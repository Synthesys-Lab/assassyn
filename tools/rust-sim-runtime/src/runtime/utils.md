# Utility Functions

## Exposed Interfaces

- `cyclize(stamp: usize)`: This function provides a fixed-point style
   for the stamp value, where the last two digits are fractional,
   e.g., `1250` represents `12.50`, which is useful for time-stamped logging.
- `load_hex_file<T: Num>(array: &mut Vec<T>, init_file: &str)`: This function
  loads hexadecimal values from a specified file into the given vector.
