pub struct Data {
  name: String,
  bits: usize,
}

enum Opcode {
  Add,
  Mul,
}

enum Operand {
  Data(Data),
  Expr(Expr),
}

pub struct Expr {
  opcode: Opcode,
  operands: Vec<Box<Operand>>
}

trait Arithmetic {
  fn add(&self, other: &Data) -> Expr;
  fn mul(&self, other: &Data) -> Expr;
}

impl Data {
  pub fn new(name: &str, bits: usize) -> Self {
    Data {
      name: name.to_string(),
      bits,
    }
  }

  // pub fn add(&self, other: &Data) -> Expr {
  //   Expr {
  //     opcode: Opcode::Add,
  //     operands: vec![
  //       Box::new(),
  //       Box::new(),
  //     ]
  //   }
  // }

}

