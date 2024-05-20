use std::ops::RangeInclusive;

use crate::builder::{PortInfo, SysBuilder};
use crate::ir::node::*;
use crate::ir::*;

use super::Attribute;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryParams {
  pub width: usize,
  pub depth: usize,
  pub lat: RangeInclusive<usize>,
  pub init_file: Option<String>,
}

impl Default for MemoryParams {
  fn default() -> Self {
    Self {
      width: 0,
      depth: 0,
      lat: 0..=0,
      init_file: None,
    }
  }
}

impl MemoryParams {
  pub fn new(
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
  ) -> Self {
    Self {
      width,
      depth,
      lat,
      init_file,
    }
  }
}

impl ToString for MemoryParams {
  fn to_string(&self) -> String {
    format!(
      "width: {} depth: {} lat: [{:?}], file: {}",
      self.width,
      self.depth,
      self.lat,
      self.init_file.clone().map_or("None".to_string(), |x| x)
    )
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
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
  ) -> BaseNode {
    let ty = DataType::Bits(width);
    let ports = vec![
      PortInfo::new("addr", DataType::UInt(depth.ilog2() as usize)),
      PortInfo::new("write", DataType::Bits(1)),
      PortInfo::new("wdata", ty.clone()),
      PortInfo::new("r", DataType::Module(vec![ty.clone().into()])),
    ];

    let module_name = self.symbol_table.identifier(name);
    let module_node = self.create_module(&module_name, ports);

    let param = MemoryParams::new(width, depth, lat, init_file);
    module_node
      .as_mut::<Module>(self)
      .unwrap()
      .add_attr(Attribute::Memory(param));

    self.set_current_module(module_node);
    let module = module_node.as_ref::<Module>(self).unwrap();
    let addr = module.get_port(0).unwrap().upcast();
    let write = module.get_port(1).unwrap().upcast();
    let wdata = module.get_port(2).unwrap().upcast();
    let r = module.get_port(3).unwrap().upcast();

    let addr = self.create_fifo_pop(addr);
    let write = self.create_fifo_pop(write);
    let wdata = self.create_fifo_pop(wdata);
    let r = self.create_fifo_pop(r);

    let buffer_name = self.symbol_table.identifier(&format!("{}_buffer", name));
    let array = self.create_array(ty, &buffer_name, depth, None);

    let ptr = self.create_array_ptr(array, addr);
    let write_block = self.create_block(BlockKind::Condition(write));

    {
      self.set_current_block(write_block);
      self.create_array_write(ptr, wdata);
    }

    let read_data = self.create_array_read(ptr);
    let data = self.create_select(write, wdata, read_data);
    let bind = self.get_init_bind(r);
    let bind = self.push_bind(bind, data, false.into());
    self.create_async_call(bind);

    module_node
  }
}
