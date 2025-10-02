"""Runtime code generation for Assassyn simulator."""

def dump_ramulator(fd):
    """Generate the ramulator module."""
    # Add memory interface part
    fd.write("""
use libloading::os::unix::{Library, Symbol};
use std::error::Error;
use std::ffi::{c_char, c_float, c_longlong, c_void, CString};

#[repr(C)]
pub struct Request {
  pub addr: i64,
  pub addr_vec: Vec<i32>,
  pub type_id: i32,
  pub source_id: i32,
  pub command: i32,
  pub final_command: i32,
  pub is_stat_updated: bool,
  pub arrive: i64,
  pub depart: i64,
  pub scratchpad: [i32; 4],
  pub callback: Option<extern "C" fn(*mut Request)>,
  pub m_payload: *mut c_void,
}

type MyWrapper = *mut c_void;
type RequestCallback = extern "C" fn(*mut Request, *mut c_void);

pub struct MemoryInterface {
  lib: Library,
  wrapper: MyWrapper,
}

impl MemoryInterface {
  pub unsafe fn new(lib: Library) -> Result<Self, Box<dyn Error>> {
    let dram_new: Symbol<unsafe extern "C" fn() -> MyWrapper> = lib.get(b"dram_new")?;
    let wrapper = dram_new();

    Ok(Self {
      lib,
      wrapper,
    })
  }

  pub unsafe fn init(&self, config_path: &str) {
    let c_path = CString::new(config_path).unwrap();
    let dram_init: Symbol<unsafe extern "C" fn(MyWrapper, *const c_char)> = self.lib.get(b"dram_init").unwrap();
    dram_init(self.wrapper, c_path.as_ptr());
  }

  pub unsafe fn frontend_tick(&self) {
    let frontend_tick: Symbol<unsafe extern "C" fn(MyWrapper)> = self.lib.get(b"frontend_tick").unwrap();
    frontend_tick(self.wrapper);
  }

  pub unsafe fn memory_tick(&self) {
    let memory_system_tick: Symbol<unsafe extern "C" fn(MyWrapper)> = self.lib.get(b"memory_system_tick").unwrap();
    memory_system_tick(self.wrapper);
  }

  pub unsafe fn send_request(
    &self,
    addr: i64,
    is_write: bool,
    callback: RequestCallback,
    ctx: *mut c_void,
  ) -> bool {
    let send_request: Symbol<unsafe extern "C" fn(MyWrapper, i64, bool, RequestCallback, *mut c_void) -> bool> = self.lib.get(b"send_request").unwrap();
    send_request(self.wrapper, addr, is_write, callback, ctx)
  }

  pub unsafe fn finish(&self) {
    let MyWrapper_finish: Symbol<unsafe extern "C" fn(MyWrapper)> = self.lib.get(b"MyWrapper_finish").unwrap();
    MyWrapper_finish(self.wrapper);
  }
}

impl Drop for MemoryInterface {
  fn drop(&mut self) {
    unsafe {
      let dram_delete: Symbol<unsafe extern "C" fn(MyWrapper)> = self.lib.get(b"dram_delete").unwrap();
      dram_delete(self.wrapper);
    }
  }
}

    """)

def dump_runtime(fd):
    """Generate the runtime module."""
    # Add imports
    fd.write("""
use std::collections::VecDeque;
use std::collections::BTreeMap;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use num_traits::Num;
use std::fs::read_to_string;
use libloading::{Library, Symbol};
use std::ffi::{c_char, c_float, c_longlong, c_void, CString};
use std::error::Error;
use std::fmt::Debug;
    """)

    # Add runtime types and implementations
    fd.write("""
pub trait Cycled {
  fn cycle(&self) -> usize;
  fn pusher(&self) -> &'static str;
}

pub struct ArrayWrite<T: Sized + Default + Clone> {
  cycle: usize,
  addr: usize,
  data: T,
  pusher: &'static str,
  port_id: usize,  // Unique identifier for the write port
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

impl <T: Sized + Default + Clone> PortXEQ<T> {
  pub fn new() -> Self {
    PortXEQ { q: BTreeMap::new() }
  }

  pub fn push(&mut self, event: ArrayWrite<T>) {
    self.q.entry(event.cycle)
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

impl <T: Sized + Default + Clone> Array<T> {
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

impl <T: Sized> FIFOPush<T> {
  pub fn new(cycle: usize, data: T, pusher: &'static str) -> Self {
    FIFOPush { cycle, data, pusher }
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

impl <T: Sized> FIFO<T> {
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

impl <T: Sized + Default + Clone> Cycled for ArrayWrite<T> {
  fn cycle(&self) -> usize {
    self.cycle
  }
  fn pusher(&self) -> &'static str {
    self.pusher
  }
}

impl <T: Sized> Cycled for FIFOPush<T> {
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

impl <T: Sized + Cycled>XEQ<T> {
  pub fn new() -> Self {
    XEQ { q: BTreeMap::new(), }
  }

  pub fn push(&mut self, event: T) {
    if let Some(a) = self.q.get(&event.cycle()) {
      panic!("{}: Already occupied by {}, cannot accept {}!",
        cyclize(a.cycle()), a.pusher(), event.pusher());
    } else {
      self.q.insert(event.cycle(), event);
    }
  }

  pub fn pop(&mut self, current: usize) -> Option<T> {
    if self.q.first_key_value().map_or(false, |(cycle, _)| *cycle <= current) {
      self.q.pop_first().map(|(_, event)| event)
    } else {
      None
    }
  }
}
    """)

    # Add utility functions
    fd.write("""
pub fn cyclize(stamp: usize) -> String {
  format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
}

pub fn load_hex_file<T: Num>(array: &mut Vec<T>, init_file: &str) {
  let mut idx = 0;
  for line in read_to_string(init_file)
    .expect("can not open hex file")
    .lines()
  {
    let line = if let Some(to_strip) = line.find("//") {
      line[..to_strip].trim()
    } else {
      line.trim()
    };
    if line.len() == 0 {
      continue;
    }
    let line = line.replace("_", "");
    if line.starts_with("@") {
      let addr = usize::from_str_radix(&line[1..], 16).unwrap();
      idx = addr;
      continue;
    }
    array[idx] = T::from_str_radix(line.as_str(), 16).ok().unwrap();
    idx += 1;
  }
}

pub trait ValueCastTo<T> {
  fn cast(&self) -> T;
}
    """)

    # Generate type casting implementations
    fd.write("impl ValueCastTo<bool> for bool { " +
              "fn cast(&self) -> bool { self.clone() } }\n")

    bigints = ["BigInt", "BigUint"]
    for i in range(2):
        bigint = bigints[i]
        other = bigints[1 - i]

        # Self cast
        fd.write(f"impl ValueCastTo<{bigint}> for {bigint} {{ " +
                  f"fn cast(&self) -> {bigint} {{ self.clone() }} }}\n")

        # Cross cast between BigInt and BigUint
        fd.write(f"""impl ValueCastTo<{other}> for {bigint} {{
          fn cast(&self) -> {other} {{ self.to_{other.lower()}().unwrap() }}
        }}\n""")

        # Bool to BigInt/BigUint
        fd.write(f"""impl ValueCastTo<{bigint}> for bool {{
          fn cast(&self) -> {bigint} {{
            if *self {{
              1.to_{bigint.lower()}().unwrap()
            }} else {{
              0.to_{bigint.lower()}().unwrap()
            }}
          }}
        }}\n""")

        # BigInt/BigUint to bool
        fd.write(f"""impl ValueCastTo<bool> for {bigint} {{
          fn cast(&self) -> bool {{
            !self.eq(&0.to_{bigint.lower()}().unwrap())
          }}
        }}\n""")

    # Generate integer casting implementations
    for sign_i in range(2):
        for i in range(3, 7):
            src_ty = f"{'ui'[sign_i]}{1 << i}"

            # To bool
            fd.write(f"impl ValueCastTo<bool> for {src_ty} {{ " +
                      "fn cast(&self) -> bool { *self != 0 } }\n")

            # From bool
            fd.write(f"""impl ValueCastTo<{src_ty}> for bool {{
                fn cast(&self) -> {src_ty} {{
                  if *self {{ 1 }} else {{ 0 }}
                }}
              }}\n""")

            # To BigInt/BigUint
            for bigint in bigints:
                fd.write(f"""impl ValueCastTo<{bigint}> for {src_ty} {{
                  fn cast(&self) -> {bigint} {{ self.to_{bigint.lower()}().unwrap() }}
                }}\n""")

            # From BigInt
            fd.write(f"""impl ValueCastTo<{src_ty}> for BigInt {{
                fn cast(&self) -> {src_ty} {{
                  let (sign, data) = self.to_u64_digits();
                  if data.is_empty() {{
                    return 0;
                  }}
                  match sign {{
                    num_bigint::Sign::Plus => data[0] as {src_ty},
                    num_bigint::Sign::Minus =>
                      ((!data[0] + 1) & ({src_ty}::MAX as u64)) as {src_ty},
                    num_bigint::Sign::NoSign => data[0] as {src_ty},
                  }}
                }}
              }}\n""")

            # From BigUint
            fd.write(f"""impl ValueCastTo<{src_ty}> for BigUint {{
                fn cast(&self) -> {src_ty} {{
                  let data = self.to_u64_digits();
                  if data.is_empty() {{
                    return 0;
                  }} else {{
                    return data[0] as {src_ty};
                  }}
                }}
              }}\n""")

            # Between integer types
            for sign_j in range(2):
                for j in range(3, 7):
                    dst_ty = f"{'ui'[sign_j]}{1 << j}"

                    if i == j and sign_i == sign_j:
                        # Self cast
                        fd.write(f"impl ValueCastTo<{dst_ty}> for {src_ty} {{ " +
                                  f"fn cast(&self) -> {dst_ty} {{ self.clone() }} }}\n")
                    else:
                        # Cross cast
                        fd.write(f"impl ValueCastTo<{dst_ty}> for {src_ty} {{ " +
                                  f"fn cast(&self) -> {dst_ty} {{ *self as {dst_ty} }} }}\n")

    # End file with newline
    fd.write("\n")
    return True
