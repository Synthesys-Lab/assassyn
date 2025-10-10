use std::collections::VecDeque;
use std::error::Error;
use std::ffi::{c_char, c_void, CString};
use std::fs;
use std::path::Path;

// Platform-specific libloading imports
#[cfg(target_os = "macos")]
use libloading::os::unix::{Library, Symbol, RTLD_GLOBAL, RTLD_LAZY};

#[cfg(target_os = "linux")]
use libloading::{Library, Symbol};

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use libloading::{Library, Symbol};

// Platform-specific library loading macro
#[cfg(target_os = "macos")]
macro_rules! load_library {
  ($path:expr) => {{
    let path_str = $path.to_string();
    let lib_path = if path_str.ends_with(".dylib") {
      path_str
    } else {
      format!("{}.dylib", $path)
    };
    unsafe { Library::open(Some(&lib_path), RTLD_GLOBAL | RTLD_LAZY)? }
  }};
}

#[cfg(target_os = "linux")]
macro_rules! load_library {
  ($path:expr) => {{
    let path_str = $path.to_string();
    let lib_path = if path_str.ends_with(".so") {
      path_str
    } else {
      format!("{}.so", $path)
    };
    unsafe { Library::new(&lib_path)? }
  }};
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
macro_rules! load_library {
  ($path:expr) => {{
    let path_str = $path.to_string();
    let lib_path = if path_str.ends_with(".dll") {
      path_str
    } else {
      format!("{}.dll", $path)
    };
    unsafe { Library::new(&lib_path)? }
  }};
}

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

#[repr(C)]
pub struct Response {
  pub valid: bool,
  pub addr: usize,
  pub data: Vec<u8>,
  pub read_succ: bool,
  pub write_succ: bool,
  pub is_write: bool,
}
type CRamualator2Wrapper = *mut c_void;
pub type RequestCallback = extern "C" fn(*mut Request, *mut c_void);

pub struct MemoryInterface {
  lib: Library,
  wrapper: CRamualator2Wrapper,
  pub write_buffer: VecDeque<(usize, Vec<u8>)>,
}

impl MemoryInterface {
  /// Create a new MemoryInterface from a loaded library.
  ///
  /// # Safety
  ///
  /// The library must be valid and contain the required symbols.
  pub unsafe fn new(lib: Library) -> Result<Self, Box<dyn Error>> {
    let dram_new: Symbol<unsafe extern "C" fn() -> CRamualator2Wrapper> = lib.get(b"dram_new")?;
    let wrapper = dram_new();

    Ok(Self {
      lib,
      wrapper,
      write_buffer: VecDeque::new(),
    })
  }

  /// Initialize the memory interface with a configuration file.
  ///
  /// # Safety
  ///
  /// The config_path must be a valid null-terminated string.
  pub unsafe fn init(&self, config_path: &str) {
    let c_path = CString::new(config_path).unwrap();
    let dram_init: Symbol<unsafe extern "C" fn(CRamualator2Wrapper, *const c_char)> =
      self.lib.get(b"dram_init").unwrap();
    dram_init(self.wrapper, c_path.as_ptr());
  }

  /// Advance the frontend by one tick.
  ///
  /// # Safety
  ///
  /// The wrapper must be in a valid state.
  pub unsafe fn frontend_tick(&self) {
    let frontend_tick: Symbol<unsafe extern "C" fn(CRamualator2Wrapper)> =
      self.lib.get(b"frontend_tick").unwrap();
    frontend_tick(self.wrapper);
  }

  /// Advance the memory system by one tick.
  ///
  /// # Safety
  ///
  /// The wrapper must be in a valid state.
  pub unsafe fn memory_system_tick(&self) {
    let memory_system_tick: Symbol<unsafe extern "C" fn(CRamualator2Wrapper)> =
      self.lib.get(b"memory_system_tick").unwrap();
    memory_system_tick(self.wrapper);
  }

  /// Get the memory clock period.
  ///
  /// # Safety
  ///
  /// The wrapper must be in a valid state.
  #[allow(non_snake_case)]
  pub unsafe fn get_memory_tCK(&self) -> f32 {
    let get_memory_tck: Symbol<unsafe extern "C" fn(CRamualator2Wrapper) -> f32> =
      self.lib.get(b"get_memory_tCK").unwrap();
    get_memory_tck(self.wrapper)
  }

  /// Send a memory request.
  ///
  /// # Safety
  ///
  /// The callback and ctx must be valid for the duration of the request.
  pub unsafe fn send_request(
    &self,
    addr: i64,
    is_write: bool,
    callback: RequestCallback,
    ctx: *mut c_void,
  ) -> bool {
    let send_request: Symbol<
      unsafe extern "C" fn(CRamualator2Wrapper, i64, bool, RequestCallback, *mut c_void) -> bool,
    > = self.lib.get(b"send_request").unwrap();
    send_request(self.wrapper, addr, is_write, callback, ctx)
  }

  /// Finish the memory interface.
  ///
  /// # Safety
  ///
  /// The wrapper must be in a valid state.
  pub unsafe fn finish(&self) {
    let my_finish: Symbol<unsafe extern "C" fn(CRamualator2Wrapper)> =
      self.lib.get(b"finish").unwrap();
    my_finish(self.wrapper);
  }

  /// Reset the write buffer and response state.
  pub fn reset_state(&mut self) {
    self.write_buffer.clear();
  }

  /// Add data to the write buffer for a write request.
  pub fn add_write_data(&mut self, addr: usize, data: Vec<u8>) {
    self.write_buffer.push_back((addr, data));
  }

  /// Get write data for a given address and remove it from the buffer.
  pub fn get_write_data(&mut self, addr: usize) -> Option<Vec<u8>> {
    if let Some(pos) = self.write_buffer.iter().position(|(a, _)| *a == addr) {
      Some(self.write_buffer.remove(pos).unwrap().1)
    } else {
      None
    }
  }
}

impl Drop for MemoryInterface {
  fn drop(&mut self) {
    unsafe {
      let dram_delete: Symbol<unsafe extern "C" fn(CRamualator2Wrapper)> =
        self.lib.get(b"dram_delete").unwrap();
      dram_delete(self.wrapper);
    }
  }
}

/// Read library path from a file generated by CMake
fn read_lib_path_from_file(file_path: &str) -> Result<String, Box<dyn Error>> {
  let path = Path::new(file_path);
  if !path.exists() {
    return Err(format!("Library path file not found: {}", file_path).into());
  }
  let content = fs::read_to_string(path)?;
  Ok(content.trim().to_string())
}

/// Get the C wrapper library path from the generated file
pub fn cwrapper_lib_path() -> Result<String, Box<dyn Error>> {
  let home = std::env::var("ASSASSYN_HOME").unwrap_or_else(|_| {
    std::env::current_dir()
      .unwrap()
      .to_string_lossy()
      .to_string()
  });
  let path_file = format!("{}/tools/c-ramulator2-wrapper/build/.cwrapper-lib-path", home);
  read_lib_path_from_file(&path_file)
}

/// Get the Ramulator2 library path from the generated file
pub fn ramulator2_lib_path() -> Result<String, Box<dyn Error>> {
  let home = std::env::var("ASSASSYN_HOME").unwrap_or_else(|_| {
    std::env::current_dir()
      .unwrap()
      .to_string_lossy()
      .to_string()
  });
  let path_file = format!("{}/tools/c-ramulator2-wrapper/build/.ramulator2-lib-path", home);
  read_lib_path_from_file(&path_file)
}

// Platform-independent constructor for MemoryInterface
impl MemoryInterface {
  /// Create a new MemoryInterface with platform-specific library loading
  pub fn new_from_path(lib_path: &str) -> Result<Self, Box<dyn Error>> {
    let lib = load_library!(lib_path);
    unsafe { Self::new(lib) }
  }

  /// Create a new MemoryInterface using the C wrapper library path from file
  pub fn new_from_cwrapper_path() -> Result<Self, Box<dyn Error>> {
    let lib_path = cwrapper_lib_path()?;
    Self::new_from_path(&lib_path)
  }
}
