use std::io::Write;

use quote::quote;

pub(super) fn dump_runtime(fd: &mut std::fs::File) {
  let mut res = String::new();
  res.push_str(
    &quote::quote! {
      use std::collections::VecDeque;
      use std::collections::BTreeMap;
      use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
      use num_traits::Num;
      use std::fs::read_to_string;
    }
    .to_string(),
  );

  let runtime = quote! {
    pub trait Cycled {
      fn cycle(&self) -> usize;
      fn pusher(&self) -> String;
    }

    pub struct ArrayWrite<T: Sized + Default + Clone> {
      cycle: usize,
      addr: usize,
      data: T,
      pusher: String,
    }

    impl <T: Sized + Default + Clone> ArrayWrite<T> {
      pub fn new(cycle: usize, addr: usize, data: T, pusher: String) -> Self {
        ArrayWrite { cycle, addr, data, pusher }
      }
    }

    pub struct Array<T: Sized + Default + Clone> {
      pub payload: Vec<T>,
      pub write: XEQ<ArrayWrite<T>>,
    }

    impl <T: Sized + Default + Clone> Array<T> {
      pub fn new(n: usize) -> Self {
        Array {
          payload: vec![T::default(); n],
          write: XEQ::new(),
        }
      }
      pub fn new_with_init(payload: Vec<T>) -> Self {
        Array {
          payload,
          write: XEQ::new(),
        }
      }
      pub fn tick(&mut self, cycle: usize) {
        if let Some(event) = self.write.pop(cycle) {
          self.payload[event.addr] = event.data;
        }
      }
    }

    pub struct FIFOPush<T: Sized> {
      cycle: usize,
      data: T,
      pusher: String,
    }

    impl <T: Sized> FIFOPush<T> {
      pub fn new(cycle: usize, data: T, pusher: String) -> Self {
        FIFOPush { cycle, data, pusher }
      }
    }

    pub struct FIFOPop {
      cycle: usize,
      pusher: String,
    }

    impl FIFOPop {
      pub fn new(cycle: usize, pusher: String) -> Self {
        FIFOPop { cycle, pusher }
      }
    }

    pub struct FIFO<T: Sized> {
      pub payload: VecDeque<T>,
      pub push: XEQ<FIFOPush<T>>,
      pub pop: XEQ<FIFOPop>,
    }

    impl <T: Sized> FIFO<T> {
      pub fn new() -> Self {
        FIFO {
          payload: VecDeque::new(),
          push: XEQ::new(),
          pop: XEQ::new(),
        }
      }

      pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
      }

      pub fn front(&self) -> Option<&T> {
        self.payload.front()
      }

      pub fn tick(&mut self, cycle: usize) {
        if let Some(_) = self.pop.pop(cycle) {
          self.payload.pop_front().unwrap();
        }
        if let Some(event) = self.push.pop(cycle) {
          self.payload.push_back(event.data);
        }
      }
    }

    impl <T: Sized + Default + Clone> Cycled for ArrayWrite<T> {
      fn cycle(&self) -> usize {
        self.cycle
      }
      fn pusher(&self) -> String {
        self.pusher.clone()
      }
    }

    impl <T: Sized> Cycled for FIFOPush<T> {
      fn cycle(&self) -> usize {
        self.cycle
      }
      fn pusher(&self) -> String {
        self.pusher.clone()
      }
    }

    impl Cycled for FIFOPop {
      fn cycle(&self) -> usize {
        self.cycle
      }
      fn pusher(&self) -> String {
        self.pusher.clone()
      }
    }

    pub struct XEQ<T: Sized + Cycled> {
      q: BTreeMap<usize, T>,
    }

    impl <T: Sized + Cycled>XEQ<T> {

      pub fn new() -> Self {
        XEQ { q: BTreeMap::new(), }
      }

      pub fn push(&mut self, event: T) {
        if let Some(event) = self.q.get(&event.cycle()) {
          panic!("Cycle {}: Already occupied by {}", event.cycle(), event.pusher());
        } else {
          self.q.insert(event.cycle(), event);
        }
      }

      pub fn pop(&mut self, current: usize) -> Option<T> {
        if self.q.first_key_value().map_or(false, |(cycle, _)| *cycle >= current) {
          self.q.pop_first().map(|(_, event)| event)
        } else {
          None
        }
      }

    }
  };

  res.push_str(&runtime.to_string());

  res.push_str(
    &quote::quote! {
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
      pub trait ValueCastTo<T> {
        fn cast(&self) -> T;
      }
    }
    .to_string(),
  );
  res.push_str("impl ValueCastTo<bool> for bool { fn cast(&self) -> bool { self.clone() } }\n");

  let bigints = ["BigInt", "BigUint"];
  for i in 0..2 {
    let bigint = bigints[i];
    let other = bigints[1 - i];
    res.push_str(&format!(
      "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.clone() }} }}\n",
      bigint, bigint, bigint
    ));
    res.push_str(&format!(
      "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.to_{}().unwrap() }} }}\n",
      other,
      bigint,
      other,
      other.to_lowercase()
    ));
    res.push_str(&format!(
      "impl ValueCastTo<{}> for bool {{ fn cast(&self) -> {} {{
        if *self {{ 1.to_{}().unwrap() }} else {{ 0.to_{}().unwrap() }}
      }} }}\n",
      bigint,
      bigint,
      bigint.to_lowercase(),
      bigint.to_lowercase()
    ));
    res.push_str(&format!(
      "impl ValueCastTo<bool> for {} {{ fn cast(&self) -> bool {{
        !self.eq(&0.to_{}().unwrap())
      }} }}\n",
      bigint,
      bigint.to_lowercase()
    ));
  }

  // Dump a template based data cast so that big integers are unified in.
  for sign_i in 0..=1 {
    for i in 3..7 {
      let src_ty = format!("{}{}", ['u', 'i'][sign_i], 1 << i);
      res.push_str(&format!(
        "impl ValueCastTo<bool> for {} {{ fn cast(&self) -> bool {{ *self != 0 }} }}\n",
        src_ty
      ));
      res.push_str(&format!(
        "impl ValueCastTo<{}> for bool {{
            fn cast(&self) -> {} {{ if *self {{ 1 }} else {{ 0 }} }}
          }}\n",
        src_ty, src_ty
      ));
      for bigint in bigints {
        res.push_str(&format!(
          "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.to_{}().unwrap() }} }}\n",
          bigint,
          src_ty,
          bigint,
          bigint.to_lowercase()
        ));
      }
      res.push_str(&format!(
        "impl ValueCastTo<{}> for BigInt {{
            fn cast(&self) -> {} {{
              let (sign, data) = self.to_u64_digits();
              if data.is_empty() {{
                return 0;
              }}
              match sign {{
                num_bigint::Sign::Plus => data[0] as {},
                num_bigint::Sign::Minus => ((!data[0] + 1) & ({}::MAX as u64)) as {},
                num_bigint::Sign::NoSign => data[0] as {},
              }}
            }}
          }}\n",
        src_ty, src_ty, src_ty, src_ty, src_ty, src_ty
      ));
      res.push_str(&format!(
        "impl ValueCastTo<{}> for BigUint {{
            fn cast(&self) -> {} {{
              let data = self.to_u64_digits();
              if data.is_empty() {{
                return 0;
              }} else {{
                return data[0] as {};
              }}
            }}
          }}\n",
        src_ty, src_ty, src_ty
      ));

      for sign_j in 0..=1 {
        for j in 3..7 {
          let dst_ty = format!("{}{}", ['u', 'i'][sign_j], 1 << j);
          if i == j && sign_i == sign_j {
            res.push_str(&format!(
              "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.clone() }} }}\n",
              dst_ty, src_ty, dst_ty
            ));
          } else {
            res.push_str(&format!(
              "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ *self as {} }} }}\n",
              dst_ty, src_ty, dst_ty, dst_ty
            ));
          }
        }
      }
    }
  }
  res.push('\n');
  fd.write_all(res.as_bytes()).unwrap();
}
