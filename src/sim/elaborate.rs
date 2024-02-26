use crate::{builder::system::SysBuilder, data::Typed, expr::Opcode};

fn dump_runtime(sys: &SysBuilder) {
  println!("// Simulation runtime.");
  println!("enum Event {{");
  for module in sys.module_iter() {
    println!("  Module_{}(Vec<u64>),", module.get_name());
  }
  println!("}}\n");

  println!("fn driver_only(q: &VecDeque<Event>) -> bool {{");
  println!("  for event in q.iter() {{");
  println!("    match event {{");
  println!("      Event::Module_driver(_) => continue,");
  println!("      _ => return false,");
  println!("    }}");
  println!("  }}");
  println!("  q.is_empty()");
  println!("}}\n");

  println!("fn main() {{");
  for array in sys.array_iter() {
    println!(
      "  let mut {} = vec![0 as {}; {}];",
      array.get_name(),
      array.dtype().to_string(),
      array.get_size()
    );
  }
  println!("  let mut q: VecDeque<Event> = VecDeque::new();");
  println!("  q.push_back(Event::Module_driver(vec![]));");
  println!("  loop {{");
  println!("    let event = q.pop_front();");
  println!("    match event {{");
  for module in sys.module_iter() {
    print!(
      "      Some(Event::Module_{}(args)) => {}(&mut q, args",
      module.get_name(),
      module.get_name()
    );
    for (array, ops) in module.array_iter(sys) {
      print!(
        ", &{}{}",
        if ops.contains(&Opcode::Store) {
          "mut "
        } else {
          ""
        },
        array.get_name(),
      );
    }
    println!("),");
  }
  println!("      _ => {{");
  println!("        println!(\"Exit @{{}}:{{}}, b/c no event to simulate!\", file!(), line!());");
  println!("        break;");
  println!("      }}");
  println!("    }}");
  println!("    if driver_only(&q) {{");
  println!("      println!(\"Exit @{{}}:{{}}, b/c all driver's to simulate!\", file!(), line!());");
  println!("      break;");
  println!("    }}");
  println!("  }}");
  println!("}}\n");
}

fn dump_module(sys: &SysBuilder) {
  for module in sys.module_iter() {
    println!("// Elaborating module {}", module.get_name());
    print!(
      "fn {}(q: &mut VecDeque<Event>, args: Vec<u64>",
      module.get_name()
    );
    for (array, ops) in module.array_iter(sys) {
      print!(
        ", {}: &{}Vec<{}>",
        array.get_name(),
        if ops.contains(&Opcode::Store) {
          "mut "
        } else {
          ""
        },
        array.dtype().to_string()
      );
    }
    println!(") {{");
    for (i, arg) in module.port_iter(sys).enumerate() {
      println!(
        "  let {} = (*args.get({}).unwrap()) as {};",
        arg.get_name(),
        i,
        arg.dtype().to_string()
      );
    }
    for elem in module.expr_iter(sys) {
      println!("  {}", elem.to_string(sys));
    }
    println!("}}\n");
  }
}

pub fn elaborate(sys: &SysBuilder) {
  println!("use std::collections::VecDeque;\n");
  dump_module(sys);
  dump_runtime(sys);
}
