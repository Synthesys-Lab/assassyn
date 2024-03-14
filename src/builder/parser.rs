#[feature(concat_idents)]

#[macro_export]
macro_rules! module_builder {

  ($sys: expr, $name: ident ) => {
    fn concat_idents!($name(), builder) -> NodeBase {
      let res = sys.create_module(stringify!($name));

      res
    }
  };

}

