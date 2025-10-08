use std::collections::{BTreeMap, VecDeque};

pub trait Cycled {
  fn cycle(&self) -> usize;
  fn pusher(&self) -> &'static str;
}

pub struct ArrayWrite<T: Sized + Default + Clone> {
  cycle: usize,
  addr: usize,
  data: T,
  pusher: &'static str,
}

impl<T: Sized + Default + Clone> ArrayWrite<T> {
  pub fn new(cycle: usize, addr: usize, data: T, pusher: &'static str) -> Self {
    ArrayWrite {
      cycle,
      addr,
      data,
      pusher,
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

pub struct Array<T: Sized + Default + Clone> {
  pub payload: Vec<T>,
  // Vec-based ports for optimal performance with compile-time port indices
  write_ports: Vec<XEQ<ArrayWrite<T>>>,
}

impl<T: Sized + Default + Clone> Array<T> {
  pub fn new(n: usize) -> Self {
    Array {
      payload: vec![T::default(); n],
      write_ports: vec![],
    }
  }

  pub fn new_with_init(payload: Vec<T>) -> Self {
    Array {
      payload,
      write_ports: vec![],
    }
  }

  pub fn new_with_ports(n: usize, num_ports: usize) -> Self {
    Array {
      payload: vec![T::default(); n],
      write_ports: (0..num_ports).map(|_| XEQ::new()).collect(),
    }
  }

  pub fn new_with_init_and_ports(payload: Vec<T>, num_ports: usize) -> Self {
    Array {
      payload,
      write_ports: (0..num_ports).map(|_| XEQ::new()).collect(),
    }
  }

  // Write with port_id - direct Vec indexing for optimal performance
  pub fn write(&mut self, port_id: usize, write: ArrayWrite<T>) {
    // Grow vec if needed (for backwards compatibility with on-demand creation)
    while port_id >= self.write_ports.len() {
      self.write_ports.push(XEQ::new());
    }
    self.write_ports[port_id].push(write);
  }

  pub fn tick(&mut self, cycle: usize) {
    // Collect all writes from all ports
    let mut pending_writes = Vec::new();

    for port in self.write_ports.iter_mut() {
      while let Some(write) = port.pop(cycle) {
        pending_writes.push(write);
      }
    }

    // Apply writes - last write wins for conflicts
    let mut write_map: BTreeMap<usize, T> = BTreeMap::new();

    for write in pending_writes {
      write_map.insert(write.addr, write.data);
    }

    for (addr, data) in write_map {
      if addr < self.payload.len() {
        self.payload[addr] = data;
      }
    }
  }
}

// FIFO structures remain unchanged
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

impl<T: Sized> Cycled for FIFOPush<T> {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
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

impl Cycled for FIFOPop {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
  }
}

pub struct FIFO<T: Sized> {
  pub payload: VecDeque<T>,
  pub push: XEQ<FIFOPush<T>>,
  pub pop: XEQ<FIFOPop>,
}

impl<T: Sized> Default for FIFO<T> {
  fn default() -> Self {
    Self::new()
  }
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
    if self.pop.pop(cycle).is_some() && !self.payload.is_empty() {
      self.payload.pop_front().unwrap();
    }
    if let Some(event) = self.push.pop(cycle) {
      self.payload.push_back(event.data);
    }
  }
}

// XEQ for exclusive events per cycle
pub struct XEQ<T: Sized + Cycled> {
  q: BTreeMap<usize, T>,
}

impl<T: Sized + Cycled> Default for XEQ<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Sized + Cycled> XEQ<T> {
  pub fn new() -> Self {
    XEQ { q: BTreeMap::new() }
  }

  pub fn push(&mut self, event: T) {
    if let Some(existing) = self.q.get(&event.cycle()) {
      panic!(
        "{}: Already occupied by {}, cannot accept {}!",
        super::utils::cyclize(existing.cycle()),
        existing.pusher(),
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
      .is_some_and(|(cycle, _)| *cycle <= current)
    {
      self.q.pop_first().map(|(_, event)| event)
    } else {
      None
    }
  }
}
