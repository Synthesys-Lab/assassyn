use std::ops::RangeInclusive;

use crate::builder::{PortInfo, SysBuilder};
use crate::ir::node::*;
use crate::{created_here, ir::*};

use super::Attribute;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryParams {
  pub array: BaseNode,
  pub width: usize,
  pub depth: usize,
  pub lat: RangeInclusive<usize>,
  pub init_file: Option<String>,
}

impl Default for MemoryParams {
  fn default() -> Self {
    Self {
      array: BaseNode::unknown(),
      width: 0,
      depth: 0,
      lat: 0..=0,
      init_file: None,
    }
  }
}

impl MemoryParams {
  pub fn new(
    array: BaseNode,
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
  ) -> Self {
    Self {
      array,
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
  pub fn declare_memory(
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
    ];

    let array_name = self.symbol_table.identifier(&format!("{}.array", name));
    let array = self.create_array(ty, &array_name, depth, None, vec![]);
    let module = self.create_module(name, ports);
    let param = MemoryParams::new(array, width, depth, lat, init_file);
    module
      .as_mut::<Module>(self)
      .unwrap()
      .add_attr(Attribute::Memory(param));
    module
  }

  pub fn impl_memory<F>(&mut self, module: BaseNode, inliner: F)
  where
    F: FnOnce(&mut SysBuilder, BaseNode, BaseNode, BaseNode),
  {
    let array = if let Some(Attribute::Memory(params)) = module
      .as_ref::<Module>(self)
      .unwrap()
      .get_attrs()
      .iter()
      .filter(|x| matches!(x, Attribute::Memory(_)))
      .next()
    {
      params.array.clone()
    } else {
      panic!("Memory module should have MemoryParams attribute");
    };

    self.set_current_module(module);
    let (addr, write, wdata) = {
      let module = module.as_ref::<Module>(self).unwrap();
      let addr = module.get_port(0).unwrap().upcast();
      let write = module.get_port(1).unwrap().upcast();
      let wdata = module.get_port(2).unwrap().upcast();

      let addr = self.create_fifo_pop(addr);
      addr.as_mut::<Expr>(self).unwrap().set_name("addr".into());
      let write = self.create_fifo_pop(write);
      write.as_mut::<Expr>(self).unwrap().set_name("write".into());
      let wdata = self.create_fifo_pop(wdata);
      wdata.as_mut::<Expr>(self).unwrap().set_name("wdata".into());
      (addr, write, wdata)
    };

    let rdata = self.create_array_read(created_here!(), array, addr);

    let wblock = self.create_block(BlockKind::Condition(write));
    self.set_current_block(wblock);
    self.create_array_write(created_here!(), array, addr, wdata);
    let new_ip = self.get_current_ip().next(self).unwrap();
    self.set_current_ip(new_ip);

    inliner(self, module, write, rdata);
  }

  /// Create a new memory, memory is a kind of special builtin modules.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs' information to the module. Refer to `PortInfo` for more details.
  pub fn create_memory<F>(
    &mut self,
    name: &str,
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
    inliner: F,
  ) -> BaseNode
  where
    F: FnOnce(&mut SysBuilder, BaseNode, BaseNode, BaseNode),
  {
    let module = self.declare_memory(name, width, depth, lat, init_file);
    self.impl_memory(module, inliner);
    module
  }
}
