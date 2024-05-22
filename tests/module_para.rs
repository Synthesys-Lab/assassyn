#[cfg(test)]
mod module_para {
    use assassyn::test_para_paser;


  #[test]
  pub fn module_para() {
    use assassyn::module_builder;
    use eir::builder::SysBuilder;

    /* module foo: raw_params
    TokenStream [
      Ident { ident: "arr1", span: #0 bytes(165..169) },
      Punct { ch: ',', spacing: Alone, span: #0 bytes(169..170) },
      Ident { ident: "arr2", span: #0 bytes(171..175) }
    ]
     */

    // module_builder!(
    //     foo(arr1, arr2) () {
    //         log("{}!", "I am foo, I take an array");
    //     }
    // );

    /* module bar: raw_params
    TokenStream [
      Ident { ident: "foo", span: #0 bytes(283..286) }
    ]
     */
    // module_builder!(
    //     bar(foo) () {
    //         log("{}!", "I am bar, I take foo");
    //     }
    // );

    /*
      module_builder!(
        foo (arr1: [int<1>; 1], arr2: [int<1>; 1]) () {
          ...
        }
      )
    */

    test_para_paser!(
      foo (arr1: [int<32>; 16], arr2: [int<1>; 1], bar: [uint<2>; 2]) () {
      }
    )
    // let mut sys = SysBuilder::new("foo_bar");
    // let arr1 = sys.create_array(eir::ir::DataType::Int(1), "arr", 1, None);
    // let arr2 = sys.create_array(eir::ir::DataType::Int(1), "arr", 1, None);

    // NOTE:
    // The parameter can be: Array, and module at first
    // Let's say for Array: it will be declared as: `name-of-array: [type<bit-length>; array-length]`
    // For modules: it shall be declared as: `name-of-module: module`

    // TODO:
    // Use ParaDecl in node.rs as the Parser template for the parameter declaration
    // ParaDecl is derive from PortDecl, which is Ident id + Dtype ty
    // So that the ParaDecl should also take the Ident item as the id
    // But for the type, it should use another declaring type, and another parse function in expr.rs
    
    // TODO:
    // By implement these, we should have same work flow of the port declare and para declare in lib.rs
    // and we should verify the type and metadata in there as well (TO BE DECIDED)


    // println!("{:?}", arr1.get_kind());
    // println!("{:?}", arr2.get_kind());

    // let _foo = foo_builder(&mut sys, arr1, arr2);
    // println!("{:?}", arr1.get_dtype(&mut sys));
    // let _bar = bar_builder(&mut sys, _foo);
  }
}
