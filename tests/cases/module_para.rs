pub fn module_para() {
  use eda4eda::module_builder;
  use eir::builder::SysBuilder;

  module_builder!(
    foo(arr) () {
			log("{}!", "I am foo, I take an array");
    }
  );

  module_builder!(
    bar(foo) () {
			log("{}!", "I am bar, I take foo");
    }
  );
	
	// the final effect
	/*
		module_builder!(
			foo (arr: lock: [int<32>; 1]) () {
				...
			}
		)	
	
	
	 */
	let mut sys = SysBuilder::new("foo_bar");
	let arr = sys.create_array(eir::ir::DataType::Int(1), "arr", 1, None);
	println!("{}", sys);
	println!("{:?}", arr);

	println!("--------------------");

	let foo =	foo_builder(&mut sys, arr);
	println!("{}", sys);

	println!("--------------------");

	bar_builder(&mut sys, foo);
	println!("{}", sys);

}

