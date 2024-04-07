use crate::builder::SysBuilder;

pub(in crate::sim) fn type_used(sys: &SysBuilder) {
  for array in sys.array_iter() {
    array.scalar_ty();
  }
}
