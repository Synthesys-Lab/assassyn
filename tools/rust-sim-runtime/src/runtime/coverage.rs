use std::collections::BTreeMap;
use std::fs;
use std::io;

#[derive(Clone, Debug)]
struct CoverageObject {
  kind: String,
  module: String,
  port: Option<String>,
  loc: Option<String>,
  expr: Option<String>,
}

/// Records source-level Assassyn semantic coverage during generated simulation.
#[derive(Clone, Debug)]
pub struct CoverageRecorder {
  roi_start: Option<usize>,
  roi_end: Option<usize>,
  objects: BTreeMap<String, CoverageObject>,
  counters: BTreeMap<String, BTreeMap<String, usize>>,
  fifo_occupancy: BTreeMap<String, isize>,
}

impl CoverageRecorder {
  /// Create a recorder that counts cycles inside the optional inclusive ROI.
  pub fn new(roi_start: Option<usize>, roi_end: Option<usize>) -> Self {
    CoverageRecorder {
      roi_start,
      roi_end,
      objects: BTreeMap::new(),
      counters: BTreeMap::new(),
      fifo_occupancy: BTreeMap::new(),
    }
  }

  /// Record a module-level event such as eligibility, fire, or blocking.
  pub fn record_module(&mut self, id: &str, module: &str, event: &str, cycle: usize) {
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "module", module, None, None, None);
    self.bump(id, event, 1);
  }

  /// Record a wait condition outcome.
  pub fn record_wait(&mut self, id: &str, module: &str, condition: bool, cycle: usize) {
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "wait", module, None, None, None);
    self.bump(id, if condition { "true" } else { "false" }, 1);
    if !condition {
      self.set_min(id, "first_false_cycle", cycle);
      self.set_value(id, "last_false_cycle", cycle);
    }
  }

  /// Record an async event enqueue.
  pub fn record_async_call(&mut self, id: &str, caller: &str, callee: &str, cycle: usize) {
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "async_call", caller, Some(callee), None, None);
    self.bump(id, "call_fire", 1);
    self.bump(id, "event_enqueue", 1);
  }

  /// Record a FIFO push and update replayed occupancy.
  pub fn record_fifo_push(
    &mut self,
    id: &str,
    module: &str,
    port: &str,
    cycle: usize,
    configured_depth: usize,
  ) {
    let current = {
      let occupancy = self.fifo_occupancy.entry(id.to_string()).or_insert(0);
      *occupancy += 1;
      (*occupancy).max(0) as usize
    };
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "fifo", module, Some(port), None, None);
    self.bump(id, "push", 1);
    self.set_value(id, "configured_rtl_depth", configured_depth);
    self.set_max(id, "max_occupancy", current);
    self.set_value(id, "final_occupancy", current);
    if current > configured_depth {
      self.bump(id, "overflow_under_configured_depth", 1);
    }
  }

  /// Record a FIFO pop and update replayed occupancy.
  pub fn record_fifo_pop(&mut self, id: &str, module: &str, port: &str, cycle: usize) {
    let (empty_pop, current) = {
      let occupancy = self.fifo_occupancy.entry(id.to_string()).or_insert(0);
      if *occupancy == 0 {
        (true, 0)
      } else {
        *occupancy -= 1;
        (false, (*occupancy).max(0) as usize)
      }
    };
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "fifo", module, Some(port), None, None);
    self.bump(id, "pop", 1);
    if empty_pop {
      self.bump(id, "empty_pop_attempt", 1);
    }
    self.set_value(id, "final_occupancy", current);
  }

  /// Record an array or register-array read.
  pub fn record_array_read(&mut self, id: &str, array: &str, cycle: usize) {
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "array", array, None, None, None);
    self.bump(id, "read", 1);
  }

  /// Record an array or register-array write.
  pub fn record_array_write(&mut self, id: &str, array: &str, cycle: usize) {
    if !self.covers(cycle) {
      return;
    }
    self.ensure_object(id, "array", array, None, None, None);
    self.bump(id, "write", 1);
  }

  /// Write the coverage JSON artifact to disk.
  pub fn flush(&self, path: &str, sim_threshold: usize) -> io::Result<()> {
    fs::write(path, self.to_json(sim_threshold))
  }

  fn covers(&self, cycle: usize) -> bool {
    if self.roi_start.is_some_and(|start| cycle < start) {
      return false;
    }
    if self.roi_end.is_some_and(|end| cycle > end) {
      return false;
    }
    true
  }

  fn ensure_object(
    &mut self,
    id: &str,
    kind: &str,
    module: &str,
    port: Option<&str>,
    loc: Option<&str>,
    expr: Option<&str>,
  ) {
    self
      .objects
      .entry(id.to_string())
      .or_insert_with(|| CoverageObject {
        kind: kind.to_string(),
        module: module.to_string(),
        port: port.map(str::to_string),
        loc: loc.map(str::to_string),
        expr: expr.map(str::to_string),
      });
  }

  fn bump(&mut self, id: &str, field: &str, amount: usize) {
    let counters = self.counters.entry(id.to_string()).or_default();
    *counters.entry(field.to_string()).or_insert(0) += amount;
  }

  fn set_value(&mut self, id: &str, field: &str, value: usize) {
    let counters = self.counters.entry(id.to_string()).or_default();
    counters.insert(field.to_string(), value);
  }

  fn set_max(&mut self, id: &str, field: &str, value: usize) {
    let counters = self.counters.entry(id.to_string()).or_default();
    let entry = counters.entry(field.to_string()).or_insert(0);
    if value > *entry {
      *entry = value;
    }
  }

  fn set_min(&mut self, id: &str, field: &str, value: usize) {
    let counters = self.counters.entry(id.to_string()).or_default();
    let entry = counters.entry(field.to_string()).or_insert(value);
    if value < *entry {
      *entry = value;
    }
  }

  fn to_json(&self, sim_threshold: usize) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema\": \"assassyn.semantic_coverage.v1\",\n");
    out.push_str("  \"roi\": {");
    out.push_str(&format!(
      "\"start_cycle\": {}, \"end_cycle\": {}",
      option_json(self.roi_start),
      option_json(self.roi_end)
    ));
    out.push_str("},\n");
    out.push_str(&format!(
      "  \"run\": {{\"sim_threshold\": {}, \"covered_cycles\": {}}},\n",
      sim_threshold,
      self.covered_cycles(sim_threshold)
    ));

    out.push_str("  \"objects\": {");
    for (index, (id, object)) in self.objects.iter().enumerate() {
      if index > 0 {
        out.push(',');
      }
      out.push('\n');
      out.push_str(&format!("    \"{}\": {{", escape_json(id)));
      out.push_str(&format!("\"kind\": \"{}\"", escape_json(&object.kind)));
      out.push_str(&format!(", \"module\": \"{}\"", escape_json(&object.module)));
      if let Some(port) = &object.port {
        out.push_str(&format!(", \"port\": \"{}\"", escape_json(port)));
      }
      if let Some(loc) = &object.loc {
        out.push_str(&format!(", \"loc\": \"{}\"", escape_json(loc)));
      }
      if let Some(expr) = &object.expr {
        out.push_str(&format!(", \"expr\": \"{}\"", escape_json(expr)));
      }
      out.push('}');
    }
    if !self.objects.is_empty() {
      out.push('\n');
      out.push_str("  ");
    }
    out.push_str("},\n");

    out.push_str("  \"counters\": {");
    for (index, (id, counters)) in self.counters.iter().enumerate() {
      if index > 0 {
        out.push(',');
      }
      out.push('\n');
      out.push_str(&format!("    \"{}\": {{", escape_json(id)));
      for (field_index, (field, value)) in counters.iter().enumerate() {
        if field_index > 0 {
          out.push_str(", ");
        }
        out.push_str(&format!("\"{}\": {}", escape_json(field), value));
      }
      out.push('}');
    }
    if !self.counters.is_empty() {
      out.push('\n');
      out.push_str("  ");
    }
    out.push_str("}\n");
    out.push_str("}\n");
    out
  }

  fn covered_cycles(&self, sim_threshold: usize) -> usize {
    let start = self.roi_start.unwrap_or(1);
    let end = self.roi_end.unwrap_or(sim_threshold);
    if end < start {
      0
    } else {
      end - start + 1
    }
  }
}

fn option_json(value: Option<usize>) -> String {
  value.map_or_else(|| "null".to_string(), |number| number.to_string())
}

fn escape_json(value: &str) -> String {
  let mut out = String::new();
  for ch in value.chars() {
    match ch {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      _ => out.push(ch),
    }
  }
  out
}
