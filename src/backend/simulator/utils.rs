use crate::ir::DataType;

pub(super) fn namify(name: &str) -> String {
  name
    .chars()
    .map(|x| {
      if x.is_alphabetic() || x.is_ascii_digit() || x == '_' {
        x
      } else {
        '_'
      }
    })
    .collect()
}

pub fn camelize(name: &str) -> String {
  let mut result = String::new();
  let mut capitalize = true;
  for c in name.chars() {
    if c == '_' {
      capitalize = true;
    } else if capitalize {
      result.push(c.to_ascii_uppercase());
      capitalize = false;
    } else {
      result.push(c);
    }
  }
  result
}

pub(super) fn dtype_to_rust_type(dtype: &DataType) -> String {
  if dtype.is_int() || dtype.is_raw() {
    let prefix = if !dtype.is_signed() || dtype.is_raw() {
      "u"
    } else {
      "i"
    };
    let bits = dtype.get_bits();
    return if (8..=64).contains(&bits) {
      let bits = bits.next_power_of_two();
      format!("{}{}", prefix, bits)
    } else if bits == 1 {
      "bool".to_string()
    } else if bits < 8 {
      format!("{}8", prefix)
    } else if bits > 64 {
      if !dtype.is_signed() || dtype.is_raw() {
        "BigUint".to_string()
      } else {
        "BigInt".to_string()
      }
    } else {
      panic!("Not implemented yet, {:?}", dtype)
    };
  }
  match dtype {
    DataType::Module(_, _) => "Box<EventKind>".to_string(),
    DataType::ArrayType(ty, size) => {
      format!("[{}; {}]", dtype_to_rust_type(ty), size)
    }
    _ => panic!("Not implemented yet, {:?}!", dtype),
  }
}
