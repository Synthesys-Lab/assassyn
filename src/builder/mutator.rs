/// The module of modifying the elements of the IR.
/// The rationale of this module is that, for such a highly interdependent data structure,
/// it is hard to borrow a single element to modify it. The borrow of this single element
/// will also lock the further borrows of the interdependent components. Therefore, a lazy
/// borrow is adopted. The mutator borrows the whole system, and the corresponding element
/// reference. Then a set of wrapper methods are provided to modify the element by the mutator.
use crate::{reference::IsElement, Reference};

use super::system::SysBuilder;


/// Register the mutator for the given type.
/// It is basically a set of deferencing methods for the mutator.
#[macro_export]
macro_rules! register_mutator {

  ($mutator: ident, $orig: ident) => {

    pub struct $mutator<'a> {
      sys: &'a mut SysBuilder,
      elem: Reference,
    }

    impl <'a> $mutator<'a> {

      pub fn get_mut(&mut self) -> &mut Box<$orig> {
        <$orig>::downcast_mut(&mut self.sys.slab, &self.elem).unwrap()
      }

      pub fn get(&self) -> &Box<$orig> {
        <$orig>::downcast(&self.sys.slab, &self.elem).unwrap()
      }

    }

    impl <'a> Mutable<'a, $orig> for $orig {
      type Mutator = $mutator<'a>;

      fn mutator(sys: &'a mut SysBuilder, elem: Reference) -> Self::Mutator {
        if let Reference::$orig(_) = elem {
          $mutator { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($orig));
        }
      }

    }

  };

}

pub trait Mutable<'a, T: IsElement<'a>> {
  type Mutator;
  fn mutator(sys: &'a mut SysBuilder, elem: Reference) -> Self::Mutator;
}

