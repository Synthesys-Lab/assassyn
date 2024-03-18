// TODO(@were): Shall we migrate these parser rules to proc-macros so that at least we have fewer
// macros to import.

#[macro_export]
macro_rules! parse_port {
  ($id:ident int $bits:literal) => {
    $crate::frontend::PortInfo::new(stringify!($id), $crate::frontend::DataType::int($bits))
  };
  ($id:ident uint $bits:literal) => {
    $crate::frontend::PortInfo::new(stringify!($id), $crate::frontend::DataType::uint($bits))
  };
}

#[macro_export]
macro_rules! parse_type {
  (int $bits:literal) => {
    $crate::frontend::DataType::int($bits)
  };
}

#[macro_export]
macro_rules! parse_idx {

  ($sys:ident [ $idx:literal ]) => {
    $sys.get_const_int(&$crate::parse_type!(int 32), $idx)
  };

  ($sys:ident $idx:ident) => {
    $idx
  };

}

#[macro_export]
macro_rules! parse_stmts {

  // when cond {
  //   <body>
  // }
  ($sys:ident when $cond:ident { $($body:tt)* } $($rest:tt)*) => {
    let ip = $sys.get_insert_point();
    let block = $sys.create_block(Some($cond.clone()));
    $sys.set_insert_point($crate::frontend::InsertPoint(ip.0.clone(), block.clone(), None));
    $crate::parse_stmts!($sys $($body)*);
    let new_at = block.as_ref::<$crate::frontend::Block>(&$sys).unwrap().next();
    $sys.set_current_block(ip.1);
    if let Some(new_at) = new_at {
      $sys.set_insert_before(&new_at);
    }
    $crate::parse_stmts!($sys $($rest)*);
  };

  // data array declaration
  // <id> = array ( <ty> "<" bits ">", size )
  ($sys:ident $id:ident = array ( $ty:ident < $bits:literal > , $size:expr ) ; $($rest:tt)*) => {
    let $id = $sys.create_array(&$crate::parse_type!($ty $bits), stringify!($id), $size, );
    $crate::parse_stmts!($sys $($rest)*);
  };

  // fifo pop
  // <id> = <fifo> . pop ( )
  ($sys:ident $dst:ident = $a:ident . pop ( ) ; $($rest:tt)*) => {
    let $dst = $sys.create_fifo_pop(&$a, None);
    $crate::parse_stmts!($sys $($rest)*);
  };

  // binary operations
  // <id> = <lhs-id> . <op> ( <rhs-literal> )
  ($sys:ident $dst:ident = $a:ident . $op:ident ( $b:literal ) ; $($rest:tt)*) => {
    let dtype = $a.get_dtype(&$sys).unwrap();
    let rhs = $sys.get_const_int(&dtype, $b);
    paste::paste! {
      let $dst = $sys.[<create_ $op>](None, &$a, &rhs);
    }
    $crate::parse_stmts!($sys $($rest)*);
  };

  // binary operations
  // <id> = <lhs-id> . <op> ( <rhs-id> )
  ($sys:ident $dst:ident = $a:ident . $op:ident ( $b:ident ) ; $($rest:tt)*) => {
    paste::paste! {
      let $dst = $sys.[<create_ $op>](None, &$a, &$b);
    }
    $crate::parse_stmts!($sys $($rest)*);
  };

  // unary operations
  // <id> = <operand-id> . <op> ( )
  ($sys:ident $dst:ident = $a:ident . $op:ident ( ) ; $($rest:tt)*) => {
    paste::paste! {
      let $dst = $sys.[<create_ $op>](&$a);
    }
    $crate::parse_stmts!($sys $($rest)*);
  };

  // array read
  // <id> = <array> [ <idx> ]
  ($sys:ident $dst:ident = $a:ident $idx:tt ; $($rest:tt)* ) => {
    paste::paste! {
      let [<$dst _idx>] = {
        let idx = $crate::parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      let $dst = $sys.create_array_read(&[<$dst _idx>]);
    }
    $crate::parse_stmts!($sys $($rest)*);
  };

  // call
  // async <func> ( <args>,* )
  ($sys:ident async $func:ident ( $($args:ident),* $(,)? ) ; $($rest:tt)* ) => {
    $sys.create_bundled_trigger(&$func, vec![$($args.clone()),*]);
    $crate::parse_stmts!($sys $($rest)*);
  };

  // spin call
  // spin <lock> <func> ( <args>,* )
  ($sys:ident spin $lock:ident [ $idx:tt ] $func:ident ( $($args:ident),* $(,)? ) ; $($rest:tt)* ) => {
    // let lock_ptr = $sys.create_array_ptr(&$lock, &0);
    // let lock = $sys.create_spin_trigger(&lock_ptr, &$func, vec![$($args.clone()),*]);
    $crate::parse_stmts!($sys $($rest)*);
  };

  // array write
  // <array> [ <idx> ] = <value-literal>
  ($sys:ident $a:ident $idx:tt = $value:literal; $($rest:tt)* ) => {
    paste::paste! {
      let [<$a _idx>] = {
        let idx = $crate::parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      let dtype = {
        let array = $a.as_ref::<frontend::Array>(&$sys).unwrap();
        array.scalar_ty().clone()
      };
      let value = $sys.get_const_int(&dtype, $value as u64);
      $sys.create_array_write(&[<$a _idx>], &value, None);
    }
    // $sys.create_index(None, $a, $idx, None);
    parse_stmts!($sys $($rest)*);
  };

  // array write
  // <array> [ <idx> ] = <value-ident>
  ($sys:ident $a:ident $idx:tt = $value:ident; $($rest:tt)* ) => {
    paste::paste! {
      let [<$a _idx>] = {
        let idx = $crate::parse_idx!($sys $idx);
        $sys.create_array_ptr(&$a, &idx)
      };
      $sys.create_array_write(&[<$a _idx>], &$value);
    }
    // $sys.create_index(None, $a, $idx, None);
    $crate::parse_stmts!($sys $($rest)*);
  };

  ($sys:ident) => {
  };

}

#[macro_export]
macro_rules! emit_ports {

  (emit $sys:ident $module:ident, $index:expr, $id:ident $($rest:ident)+ ) => {
    let module_ref = $module.as_ref::<$crate::frontend::Module>($sys).unwrap();
    let $id = module_ref.get_input($index).unwrap().clone();
    $crate::emit_ports!(emit $sys $module, $index + 1, $($rest)+);
  };

  (emit $sys:ident $module:ident, $index:expr, $id:ident) => {
    let module_ref = $module.as_ref::<$crate::frontend::Module>($sys).unwrap();
    let $id = module_ref.get_input($index).unwrap().clone();
  };

  (enter $sys:ident $module:ident, $port:ident) => {
    let $port = {
      $crate::emit_ports!(emit $sys $module, 0, $port);
      $port
    };
  };

  (enter $sys:ident $module:ident, $($ports:ident)+) => {
    let ( $($ports),+ ) = {
      $crate::emit_ports!(emit $sys $module, 0, $($ports)+);
      ( $($ports),+ )
    };
  };

  (enter $sys:ident $module:ident, ) => {
  };

}

#[macro_export]
macro_rules! emit_signature {

  (enter) => {
    $crate::frontend::BaseNode
  };

  (enter $($ret:ident),+) => {
    ( $crate::frontend::BaseNode, $crate::emit_signature!(emit $($ret),+) )
  };

  (emit $ret:ident, $($rest:ident),*) => {
    $crate::frontend::BaseNode, $crate::emit_signature!(emit $($rest),*)
  };

  (emit $ret:ident) => {
    $crate::frontend::BaseNode
  };

  (emit) => {
  }

}

#[macro_export]
macro_rules! emit_rets {

  (enter $res:ident) => {
    $res
  };

  (enter $res:ident  $($ret:ident),* ) => {
    ( $res, $($ret),* )
  };

}

#[macro_export]
macro_rules! module_builder {
  ($name:ident [$($id:ident : $ty:ident < $bits:literal >),* $(,)?] [$($ext:ident),* $(,)?] {
    $($body:tt)*
  } $(. expose( $($ret:ident),* $(,)? ))? ) => {
    paste::paste! {
      fn [<$name _builder>](sys: &mut SysBuilder, $($ext: $crate::frontend::BaseNode),*)
        -> $crate::emit_signature!(enter $( $($ret),* )?) {
        let ports = vec![$($crate::parse_port!($id $ty $bits)),*];
        let res = sys.create_module(stringify!($name), ports);
        $crate::emit_ports!(enter sys res, $($id)*);
        sys.set_current_module(&res);
        $crate::parse_stmts!( sys $($body)* );
        return $crate::emit_rets!(enter res $( $($ret),* )?)
      }
    }
  };
}
