# Exclusive Event Queue (XEQ)

The XEQ is a specialized data structure to simulate register write.
In Assassyn, we have two key types of register writes:
1. Register array writes handled by `ArrayWrite`.
2. Stage register writes handled by `FIFOPush`.

## Trait

````rust
pub trait Cycled {
  fn cycle(&self) -> usize;
  fn pusher(&self) -> &'static str;
}
````

- Each cycle each event is exclusive, so we have a `cycle` function to indicate to
  which cycle this event belongs.
- The `pusher` function indicates which module pushes this event, which is useful for debugging.

## Entry

````rust
pub struct ArrayWrite<T: Sized + Default + Clone> {
  cycle: usize,
  addr: usize,
  data: T,
  pusher: &'static str,
}

pub struct Array<T: Sized + Default + Clone> {
  pub payload: Vec<T>,
  write_ports: Vec<XEQ<ArrayWrite<T>>>,
}
````

`ArrayWrite` is used to model register array writes.
Each register file can have multiple different write ports managed through a `Vec`,
where the index is the port ID and the value is an XEQ for that port.

### Port ID Assignment

Port IDs are assigned at **compile time** using sequential integers for optimal performance:
- The Python code generator analyzes the system during elaboration
- Each module writing to an array gets a unique port index
- Port IDs are small, predictable, sequential integers
- Pre-allocated Vec provides direct O(1) indexing

### Runtime Behavior

- The `new_with_ports` and `new_with_init_and_ports` constructors pre-allocate the exact number of ports needed
- The `write` method uses direct Vec indexing with the compile-time assigned port ID
- For backwards compatibility, ports can still be created on-demand if needed
- `tick` commits all pending writes from all ports to the register array payload
- When multiple writes to the same address occur in the same cycle (from different ports),
  the last write wins

## XEQ

````rust
pub struct XEQ<T: Sized + Cycled> {
  q: BTreeMap<usize, T>,
}
````

- When pushing to `XEQ`, if there is already an event for the same cycle,
  an error will be raised.