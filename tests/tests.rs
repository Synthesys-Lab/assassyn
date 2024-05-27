mod cases;

macro_rules! register_test {

  ($module:ident :: $func:ident) => {
    #[test]
    fn $func() {
        cases::$module::$func();
    }
  };

  ($module:ident :: { $($func:ident),* $(,)? }) => {
    $(
      #[test]
      fn $func() {
          cases::$module::$func();
      }
    )*
  }

}

register_test!(array::{array_multi_read, array_multi_write_in_same_module, array_partition0, array_partition1});
register_test!(back_pressure::back_pressure);
register_test!(fifo_valid::fifo_valid);
register_test!(fib::fib);
register_test!(testbench::testbench);
register_test!(concat::concat);
register_test!(systolic::systolic_array);
register_test!(memory::sram);
register_test!(memory::sram_init);
register_test!(multi_call::multi_call);
register_test!(eager_bind::eager_bind);
register_test!(data_type_conversion::dt_conv);
register_test!(inline::{inline0, inline1});
register_test!(explicit_pop::explicit_pop);
register_test!(select::select);
register_test!(helloworld::helloworld);
register_test!(common_read::common_read);
register_test!(cond_cse::cond_cse);
register_test!(arbiter::arbiter);
register_test!(spin_lock::spin_lock);
register_test!(async_call::async_call);
register_test!(bind::bind);
