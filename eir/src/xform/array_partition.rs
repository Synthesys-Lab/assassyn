use std::collections::{HashMap, HashSet};

use crate::{
  builder::SysBuilder,
  ir::{
    data::ArrayAttr,
    instructions,
    node::{BaseNode, ExprRef, IsElement},
    visitor::Visitor,
    Array, BlockKind, Expr, IntImm, Opcode,
  },
};

struct GatherUsage {
  usage: HashMap<BaseNode, Vec<BaseNode>>, // key: array, value: list of load/store
  to_partition: HashSet<BaseNode>,
}

impl GatherUsage {
  fn new(to_partition: HashSet<BaseNode>) -> Self {
    let usage = HashMap::from_iter(to_partition.iter().map(|x| (x.clone(), vec![])));
    Self {
      usage,
      to_partition,
    }
  }
}

impl Visitor<()> for GatherUsage {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    if let Some((array, user)) = match expr.get_opcode() {
      Opcode::Load => {
        let load = expr.as_sub::<instructions::Load>().unwrap();
        Some((load.array().upcast(), load.get().upcast()))
      }
      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        Some((store.array().upcast(), store.get().upcast()))
      }
      _ => None,
    } {
      if self.to_partition.contains(&array) {
        self.usage.get_mut(&array).unwrap().push(user);
      }
    }
    None
  }
}

pub fn rewrite_array_partitions(sys: &mut SysBuilder) {
  let to_partition = sys
    .array_iter()
    .filter(|array| array.get_attrs().contains(&ArrayAttr::FullyPartitioned))
    .map(|x| x.upcast())
    .collect();

  let mut gather = GatherUsage::new(to_partition);
  gather.enter(sys);

  for array in gather.to_partition.iter() {
    let (dtype, name, size, init) = {
      let array = array.as_ref::<Array>(sys).unwrap();
      let init = array
        .get_initializer()
        .map(|x| x.iter().map(|x| vec![x.clone()]).collect::<Vec<_>>());
      (
        array.scalar_ty().clone(),
        array.get_name().to_string(),
        array.get_size(),
        init,
      )
    };
    let partition = (0..size)
      .map(|i| {
        let init_val = if let Some(init) = &init {
          Some(init[i].clone())
        } else {
          None
        };
        sys.create_array(
          dtype.clone(),
          &format!("{}.partition.{}", name, i),
          1,
          init_val,
          vec![],
        )
      })
      .collect::<Vec<_>>();
    for user in gather.usage.get(array).unwrap() {
      let (opcode, idx, value) = {
        let expr = user.as_ref::<Expr>(sys).unwrap();
        let opcode = expr.get_opcode();
        let idx = expr.get_operand(1).unwrap().get_value().clone();
        let value = expr.get_operand(2).map(|x| x.get_value().clone());
        (opcode, idx, value)
      };
      let idx_ty = idx.get_dtype(sys).unwrap();
      let zero = sys.get_const_int(idx_ty.clone(), 0);
      match opcode {
        Opcode::Load => {
          sys.set_insert_before(user.clone());
          let new_load = if let Ok(idx_imm) = idx.as_ref::<IntImm>(sys) {
            let idx = idx_imm.get_value();
            sys.create_array_read(partition[idx as usize], zero)
          } else {
            let p0 = sys.create_array_read(partition[0], zero);
            (1..size).fold(p0, |acc, x| {
              let cur = sys.get_const_int(idx_ty.clone(), x as u64);
              let value = sys.create_array_read(partition[x], zero);
              let cond = sys.create_eq(idx.clone(), cur);
              sys.create_select(cond, value, acc)
            })
          };
          sys.replace_all_uses_with(user.clone(), new_load);
        }
        Opcode::Store => {
          if let Ok(idx_imm) = idx.as_ref::<IntImm>(sys) {
            let idx = idx_imm.get_value();
            sys.create_array_write(partition[idx as usize], zero, value.unwrap());
          } else {
            (0..size).for_each(|x| {
              sys.set_insert_before(user.clone());
              let cur = sys.get_const_int(idx_ty.clone(), x as u64);
              let cond = sys.create_eq(idx.clone(), cur);
              let block = sys.create_block(BlockKind::Condition(cond));
              sys.set_current_block(block);
              sys.create_array_write(partition[x], zero, value.unwrap());
            });
          }
        }
        _ => unreachable!(),
      }
      user.as_mut::<Expr>(sys).unwrap().erase_from_parent();
    }
    sys.remove_array(array.clone());
  }
}
