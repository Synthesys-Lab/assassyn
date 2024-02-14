use crate::Reference;

pub enum EventKind {
  Spin(Reference),
  Cond(Reference),
  Trigger,
}

pub struct Event {
  /// The source module of the event
  src: Reference,
  /// The destination module of the event
  dst: Reference,
  /// Connect the data to the destination.
  data: Vec<Reference>,
  /// Condition to trigger the event
  kind: EventKind,
}


impl Event {

  pub fn new(src: Reference,
             dst: Reference,
             data: Vec<Reference>,
             kind: EventKind) -> Self {
    Self { src, dst, data, kind, }
  }

}

