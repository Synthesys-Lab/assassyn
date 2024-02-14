use crate::Reference;

struct Event {
  /// The source module of the event
  src: Reference,
  /// The destination module of the event
  dst: Reference,
  /// Connect the data to the destination.
  data: Vec<(String, Reference)>
}

