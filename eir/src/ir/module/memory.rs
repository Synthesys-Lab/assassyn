use crate::builder::{PortInfo, SysBuilder};
use crate::ir::node::*;
use crate::ir::*;

pub struct MemoryParams {
  pub name: String,
  pub width: usize,
  pub depth: usize,
  pub lat_min: usize,
  pub lat_max: usize,
  pub init_file: Option<String>,
}

pub fn module_is_memory(module_name: &String) -> bool {
  module_name.starts_with("__builtin_memory")
}

pub fn parse_memory_module_name(module_name: &String) -> Option<MemoryParams> {
  if module_is_memory(module_name) {
    let mem_name = module_name.replace("__builtin_memory_", "");
    let mem_name = mem_name.replace(".", "_");
    let mut mem_params = mem_name.split("_").collect::<Vec<_>>();
    mem_params.remove(mem_params.len() - 1);
    let mem_width = mem_params[0][1..].parse::<usize>().unwrap();
    let mem_depth = mem_params[1][1..].parse::<usize>().unwrap();
    let mem_lat_min = mem_params[2][1..].parse::<usize>().unwrap();
    let mem_lat_max = mem_params[3].parse::<usize>().unwrap();
    let mem_name = mem_params[4].to_string();
    let init_file = if mem_params.len() == 6 {
      Some(mem_params[5].to_string())
    } else {
      None
    };
    Some(MemoryParams {
      name: mem_name,
      width: mem_width,
      depth: mem_depth,
      lat_min: mem_lat_min,
      lat_max: mem_lat_max,
      init_file: init_file,
    })
  } else {
    None
  }
}

impl SysBuilder {
  /// Create a new memory, memory is a kind of special builtin modules.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs' information to the module. Refer to `PortInfo` for more details.
  pub fn create_memory(
    &mut self,
    name: &str,
    ty: DataType,
    depth: usize,
    (lat_min, lat_max): (usize, usize),
    init_file: Option<String>,
  ) -> BaseNode {
    // TODO: addr is a UInt, not Int
    let ports = vec![
      PortInfo::new("raddr", DataType::Int(depth.ilog2() as usize)),
      PortInfo::new("r", DataType::Module(vec![ty.clone().into()])),
    ];
    let name = name.replace("_", "");
    let module_name = self.identifier(&format!(
      "__builtin_memory_w{}_d{}_l{}_{}_{}{}{}",
      ty.get_bits(),
      depth,
      lat_min,
      lat_max,
      name,
      init_file.as_ref().map_or("", |_| "_"),
      init_file.unwrap_or("".to_string())
    ));
    let module_node = self.create_module(&module_name, ports);
    self.set_current_module(module_node);
    let module = module_node.as_ref::<Module>(self).unwrap();
    let r_module_fifo = module.get_port_by_name("r").unwrap();
    let r_module = self.create_fifo_pop(r_module_fifo.upcast().clone(), None);
    let bind = self.get_init_bind(r_module);
    let const_zero = self.get_const_int(ty.clone(), 0);
    let bind = self.push_bind(bind, const_zero, false);
    self.create_trigger_bound(bind);
    module_node
  }
}
