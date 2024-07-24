use crate::builder::SysBuilder;
use crate::ir::node::IsElement;
use crate::ir::Module;

pub fn erase_metadata(sys: &mut SysBuilder) {
  let modules = sys.module_iter().map(|x| x.upcast()).collect::<Vec<_>>();
  modules.iter().for_each(|module| {
    let mut module_mut = module.as_mut::<Module>(sys).unwrap();
    module_mut.set_parameterizable(vec![]);
  });
}
