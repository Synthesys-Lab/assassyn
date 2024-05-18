#[cfg(test)]
mod module_para {

  #[test]
  pub fn module_para() {
    use eda4eda::module_builder;
    use eir::builder::SysBuilder;

    /* module foo: raw_params
    TokenStream [
      Ident { ident: "arr1", span: #0 bytes(165..169) },
      Punct { ch: ',', spacing: Alone, span: #0 bytes(169..170) },
      Ident { ident: "arr2", span: #0 bytes(171..175) }
    ]
     */

    module_builder!(
        foo(arr1, arr2) () {
            log("{}!", "I am foo, I take an array");
        }
    );

    /* module bar: raw_params
    TokenStream [
      Ident { ident: "foo", span: #0 bytes(283..286) }
    ]
     */
    module_builder!(
        bar(foo) () {
            log("{}!", "I am bar, I take foo");
        }
    );

    /*
      module_builder!(
        foo (arr1: [int<1>; 1], arr2: [int<1>; 1]) () {
          ...
        }
      )
    */

    let mut sys = SysBuilder::new("foo_bar");
    let arr1 = sys.create_array(eir::ir::DataType::Int(1), "arr", 1, None);
    let arr2 = sys.create_array(eir::ir::DataType::Int(1), "arr", 1, None);

    let _foo = foo_builder(&mut sys, arr1, arr2);
    let _bar = bar_builder(&mut sys, _foo);
  }
}
