use std::collections::{BTreeMap, HashMap, VecDeque};

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
  // Use HashMap with port_id as key for simpler multi-port management
  write_ports: HashMap<usize, XEQ<ArrayWrite<T>>>,
}

impl<T: Sized + Default + Clone> Array<T> {
  pub fn new(n: usize) -> Self {
    Array {
      payload: vec![T::default(); n],
      write_ports: HashMap::new(),
    }
  }

  pub fn new_with_init(payload: Vec<T>) -> Self {
    Array {
      payload,
      write_ports: HashMap::new(),
    }
  }

  // Write with port_id - creates port on demand
  pub fn write(&mut self, port_id: usize, write: ArrayWrite<T>) {
    self
      .write_ports
      .entry(port_id)
      .or_insert_with(XEQ::new)
      .push(write);
  }

  pub fn tick(&mut self, cycle: usize) {
    // Collect all writes from all ports
    let mut pending_writes = Vec::new();

    for port in self.write_ports.values_mut() {
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

// XEQ for exclusive events per cycle
pub struct XEQ<T: Sized + Cycled> {
  q: BTreeMap<usize, T>,
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
      .map_or(false, |(cycle, _)| *cycle <= current)
    {
      self.q.pop_first().map(|(_, event)| event)
    } else {
      None
    }
  }
}
