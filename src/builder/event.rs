use crate::Reference;

pub(crate) struct EventImpl {
  /// The source module of the event
  src: Reference,
  /// The destination module of the event
  dst: Reference,
  /// Connect the data to the destination.
  data: Vec<Reference>,
  /// Condition to trigger the event
  pred: Option<Reference>,
}

pub enum Event {
  Spin(EventImpl),
  Cond(EventImpl),
  Trigger(EventImpl),
}

impl EventImpl {

  pub fn new(src: Reference,
             dst: Reference,
             data: Vec<Reference>,
             pred: Option<Reference>) -> Self {
    Self { src, dst, data, pred, }
  }

}

