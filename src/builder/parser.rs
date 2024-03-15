#[macro_export]
macro_rules! parse_port {
  ($id:ident int $bits:literal) => {
    PortInfo::new(stringify!($id), DataType::int($bits))
  };
  ($id:ident uint $bits:literal) => {
    PortInfo::new(stringify!($id), DataType::uint($bits))
  };
}

#[macro_export]
macro_rules! parse_type {
  (int $bits:literal) => {
    DataType::int($bits)
  };
}

#[macro_export]
macro_rules! parse_idx {

  ($sys:ident [ $idx:literal ]) => {
    $sys.get_const_int(&parse_type!(int 32), $idx)
  };

  ($sys:ident $idx:ident) => {
    $idx
  };

}

#[macro_export]
macro_rules! parse_stmts {

  ($sys:ident when $cond:ident { $($body:tt)* } $($rest:tt)*) => {
    let ip = $sys.get_insert_point();
    let block = $sys.create_block(Some($cond.clone()));
    $sys.set_insert_point(InsertPoint(ip.0.clone(), block.clone(), None));
    parse_stmts!($sys $($body)*);
    let new_at = block.as_ref::<Block>(&$sys).unwrap().next();
    $sys.set_current_block(ip.1);
    if let Some(new_at) = new_at {
      println!("Inserting before {:?}", new_at);
      $sys.set_insert_before(&new_at);
    }
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $id:ident = array ( $ty:ident < $bits:literal > , $size:expr ) ; $($rest:tt)*) => {
    let $id = $sys.create_array(&parse_type!($ty $bits), stringify!($id), $size, );
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $dst:ident = $a:ident . pop ( ) ; $($rest:tt)*) => {
    let $dst = $sys.create_fifo_pop(&$a, None);
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $dst:ident = $a:ident . $op:ident ( $b:literal ) ; $($rest:tt)*) => {
    let dtype = $a.get_dtype(&$sys).unwrap();
    let rhs = $sys.get_const_int(&dtype, $b);
    paste! {
      let $dst = $sys.[<create_ $op>](None, &$a, &rhs);
    }
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $dst:ident = $a:ident . $op:ident ( $b:ident ) ; $($rest:tt)*) => {
    paste! {
      let $dst = $sys.[<create_ $op>](None, &$a, &$b);
    }
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $dst:ident = $a:ident $idx:tt ; $($rest:tt)* ) => {
    paste! {
      let [<$dst _idx>] = {
        let idx = parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      let $dst = $sys.create_array_read(&[<$dst _idx>]);
    }
    // $sys.create_index(None, $a, $idx, None);
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident async $func:ident ( $($args:ident),* $(,)? ) ; $($rest:tt)* ) => {
    $sys.create_bundled_trigger(&$func, vec![$($args.clone()),*]);
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $a:ident $idx:tt = $value:literal; $($rest:tt)* ) => {
    paste! {
      let [<$a _idx>] = {
        let idx = parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      let dtype : DataType = {
        let array = $a.as_ref::<Array>(&$sys).unwrap();
        array.scalar_ty().clone()
      };
      let value = $sys.get_const_int(&dtype, $value as u64);
      $sys.create_array_write(&[<$a _idx>], &value, None);
    }
    // $sys.create_index(None, $a, $idx, None);
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident $a:ident $idx:tt = $value:ident; $($rest:tt)* ) => {
    paste! {
      let [<$a _idx>] = {
        let idx = parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      $sys.create_array_write(&[<$a _idx>], &$value);
    }
    // $sys.create_index(None, $a, $idx, None);
    parse_stmts!($sys $($rest)*);
  };

  ($sys:ident) => {
  };

}

#[macro_export]
macro_rules! emit_ports {

  (emit $sys:ident $module:ident, $index:expr, $id:ident $($rest:ident)+ ) => {
    let module_ref = $module.as_ref::<Module>($sys).unwrap();
    let $id:BaseNode = module_ref.get_input($index).unwrap().clone();
    emit_ports!(emit $sys $module, $index + 1, $($rest)+);
  };

  (emit $sys:ident $module:ident, $index:expr, $id:ident) => {
    let module_ref = $module.as_ref::<Module>($sys).unwrap();
    let $id:BaseNode = module_ref.get_input($index).unwrap().clone();
  };

  (enter $sys:ident $module:ident, $($ports:ident)+) => {
    let ( $($ports),+ ) = {
      emit_ports!(emit $sys $module, 0, $($ports)+);
      ( $($ports),+ )
    };
  };

  (enter $sys:ident $module:ident, ) => {
  };

}

#[macro_export]
macro_rules! module_builder {
  ($name:ident [$($id:ident : $ty:ident < $bits:literal >),* $(,)?] [$($ext:ident),* $(,)?] {
    $($body:tt)*
  }) => {
    paste! {
      fn [<$name _builder>](sys: &mut SysBuilder, $($ext: BaseNode),*) -> BaseNode {
        let ports = vec![$(parse_port!($id $ty $bits)),*];
        let res = sys.create_module(stringify!($name), ports);
        emit_ports!(enter sys res, $($id)*);
        sys.set_current_module(&res);
        parse_stmts!( sys $($body)* );
        return res;
      }
    }
  };
}
