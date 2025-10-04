use std::collections::BTreeMap;
use std::collections::VecDeque;

pub trait Cycled {
  fn cycle(&self) -> usize;
  fn pusher(&self) -> &'static str;
}

pub struct ArrayWrite<T: Sized + Default + Clone> {
  cycle: usize,
  addr: usize,
  data: T,
  pusher: &'static str,
  port_id: usize, // Unique identifier for the write port
}

impl<T: Sized + Default + Clone> ArrayWrite<T> {
  pub fn new(cycle: usize, addr: usize, data: T, pusher: &'static str, port_id: usize) -> Self {
    ArrayWrite {
      cycle,
      addr,
      data,
      pusher,
      port_id,
    }
  }
}

// The write queue that can handle multiple writes per cycle
pub struct PortXEQ<T: Sized + Default + Clone> {
  // Map from cycle to list of writes for that cycle
  q: BTreeMap<usize, Vec<ArrayWrite<T>>>,
}

impl<T: Sized + Default + Clone> PortXEQ<T> {
  pub fn new() -> Self {
    PortXEQ { q: BTreeMap::new() }
  }

  pub fn push(&mut self, event: ArrayWrite<T>) {
    self
      .q
      .entry(event.cycle)
      .or_insert_with(Vec::new)
      .push(event);
  }

  pub fn pop_all(&mut self, current: usize) -> Vec<ArrayWrite<T>> {
    let mut writes = Vec::new();

    // Collect all writes up to current cycle
    while let Some((&cycle, _)) = self.q.first_key_value() {
      if cycle <= current {
        if let Some((_, cycle_writes)) = self.q.pop_first() {
          writes.extend(cycle_writes);
        }
      } else {
        break;
      }
    }

    writes
  }
}

pub struct Array<T: Sized + Default + Clone> {
  pub payload: Vec<T>,
  pub write_port: PortXEQ<T>,
}

impl<T: Sized + Default + Clone> Array<T> {
  pub fn new(n: usize) -> Self {
    Array {
      payload: vec![T::default(); n],
      write_port: PortXEQ::new(),
    }
  }

  pub fn new_with_init(payload: Vec<T>) -> Self {
    Array {
      payload,
      write_port: PortXEQ::new(),
    }
  }

  pub fn tick(&mut self, cycle: usize) {
    let port_writes = self.write_port.pop_all(cycle);

    // Apply writes with conflict resolution
    // Strategy: Last write wins (could be changed to priority-based or other schemes)
    let mut write_map: BTreeMap<usize, (T, &'static str, usize)> = BTreeMap::new();

    for write in port_writes {
      write_map.insert(write.addr, (write.data, write.pusher, write.port_id));
    }

    // Apply all writes
    for (addr, (data, _, _)) in write_map {
      self.payload[addr] = data;
    }
  }
}

pub struct FIFOPush<T: Sized> {
  cycle: usize,
  data: T,
  pusher: &'static str,
}

impl<T: Sized> FIFOPush<T> {
  pub fn new(cycle: usize, data: T, pusher: &'static str) -> Self {
    FIFOPush {
      cycle,
      data,
      pusher,
    }
  }
}

pub struct FIFOPop {
  cycle: usize,
  pusher: &'static str,
}

impl FIFOPop {
  pub fn new(cycle: usize, pusher: &'static str) -> Self {
    FIFOPop { cycle, pusher }
  }
}

pub struct FIFO<T: Sized> {
  pub payload: VecDeque<T>,
  pub push: XEQ<FIFOPush<T>>,
  pub pop: XEQ<FIFOPop>,
}

impl<T: Sized> FIFO<T> {
  pub fn new() -> Self {
    FIFO {
      payload: VecDeque::new(),
      push: XEQ::new(),
      pop: XEQ::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.payload.is_empty()
  }

  pub fn front(&self) -> Option<&T> {
    self.payload.front()
  }

  pub fn tick(&mut self, cycle: usize) {
    if let Some(_) = self.pop.pop(cycle) {
      if !self.payload.is_empty() {
        self.payload.pop_front().unwrap();
      }
    }
    if let Some(event) = self.push.pop(cycle) {
      self.payload.push_back(event.data);
    }
  }
}

impl<T: Sized + Default + Clone> Cycled for ArrayWrite<T> {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
  }
}

impl<T: Sized> Cycled for FIFOPush<T> {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
  }
}

impl Cycled for FIFOPop {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
  }
}

// Single-port write queue (kept for backward compatibility)
pub struct XEQ<T: Sized + Cycled> {
  q: BTreeMap<usize, T>,
}

impl<T: Sized + Cycled> XEQ<T> {
  pub fn new() -> Self {
    XEQ { q: BTreeMap::new() }
  }

  pub fn push(&mut self, event: T) {
    if let Some(a) = self.q.get(&event.cycle()) {
      panic!(
        "{}: Already occupied by {}, cannot accept {}!",
        super::utils::cyclize(a.cycle()),
        a.pusher(),
        event.pusher()
      );
    } else {
      self.q.insert(event.cycle(), event);
    }
  }

  pub fn pop(&mut self, current: usize) -> Option<T> {
    if self
      .q
      .first_key_value()
      .map_or(false, |(cycle, _)| *cycle <= current)
    {
      self.q.pop_first().map(|(_, event)| event)
    } else {
      None
    }
  }
}
