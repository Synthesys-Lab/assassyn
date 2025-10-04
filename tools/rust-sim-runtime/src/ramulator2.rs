use libloading::os::unix::{Library, Symbol};
use std::error::Error;
use std::ffi::{c_char, c_void, CString};

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

    Ok(Self { lib, wrapper })
  }

  pub unsafe fn init(&self, config_path: &str) {
    let c_path = CString::new(config_path).unwrap();
    let dram_init: Symbol<unsafe extern "C" fn(MyWrapper, *const c_char)> =
      self.lib.get(b"dram_init").unwrap();
    dram_init(self.wrapper, c_path.as_ptr());
  }

  pub unsafe fn frontend_tick(&self) {
    let frontend_tick: Symbol<unsafe extern "C" fn(MyWrapper)> =
      self.lib.get(b"frontend_tick").unwrap();
    frontend_tick(self.wrapper);
  }

  pub unsafe fn memory_tick(&self) {
    let memory_system_tick: Symbol<unsafe extern "C" fn(MyWrapper)> =
      self.lib.get(b"memory_system_tick").unwrap();
    memory_system_tick(self.wrapper);
  }

  pub unsafe fn send_request(
    &self,
    addr: i64,
    is_write: bool,
    callback: RequestCallback,
    ctx: *mut c_void,
  ) -> bool {
    let send_request: Symbol<
      unsafe extern "C" fn(MyWrapper, i64, bool, RequestCallback, *mut c_void) -> bool,
    > = self.lib.get(b"send_request").unwrap();
    send_request(self.wrapper, addr, is_write, callback, ctx)
  }

  pub unsafe fn finish(&self) {
    let my_finish: Symbol<unsafe extern "C" fn(MyWrapper)> =
      self.lib.get(b"MyWrapper_finish").unwrap();
    my_finish(self.wrapper);
  }
}

impl Drop for MemoryInterface {
  fn drop(&mut self) {
    unsafe {
      let dram_delete: Symbol<unsafe extern "C" fn(MyWrapper)> =
        self.lib.get(b"dram_delete").unwrap();
      dram_delete(self.wrapper);
    }
  }
}
