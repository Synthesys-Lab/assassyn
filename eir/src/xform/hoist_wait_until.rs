use crate::{
  builder::SysBuilder,
  ir::{Block, BlockKind},
};

pub(super) fn hoist_wait_until(sys: &mut SysBuilder) {
  for elem in sys.module_iter() {
    let body = elem.get_body();
    match body.get_kind() {
      BlockKind::None => {
        if body.get_num_exprs() == 1 && {
          // If this is a none-block that contains a single wait-until block, hoist the wait-until
          // block to the parent block.
          body.get().get(0).map_or(false, |x| {
            if let Ok(block) = x.as_ref::<Block>(sys) {
              match block.get_kind() {
                BlockKind::WaitUntil(_) => true,
                _ => false,
              }
            } else {
              false
            }
          })
        } {}
      }
      _ => {}
    }
  }
}
