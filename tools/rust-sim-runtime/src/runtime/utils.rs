use num_traits::Num;
use std::fs::read_to_string;

pub fn cyclize(stamp: usize) -> String {
  format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
}

pub fn load_hex_file<T: Num>(array: &mut Vec<T>, init_file: &str) {
  let mut idx = 0;
  for line in read_to_string(init_file)
    .expect("can not open hex file")
    .lines()
  {
    let line = if let Some(to_strip) = line.find("//") {
      line[..to_strip].trim()
    } else {
      line.trim()
    };
    if line.len() == 0 {
      continue;
    }
    let line = line.replace("_", "");
    if line.starts_with("@") {
      let addr = usize::from_str_radix(&line[1..], 16).unwrap();
      idx = addr;
      continue;
    }
    array[idx] = T::from_str_radix(line.as_str(), 16).ok().unwrap();
    idx += 1;
  }
}
