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

````rust
pub struct ArrayWrite<T: Sized + Default + Clone> {
  cycle: usize,
  addr: usize,
  data: T,
  pusher: &'static str,
  port_id: usize, // Unique identifier for the write port
}
````