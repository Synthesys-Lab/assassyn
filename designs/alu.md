````
// op   a   b
//  |    \\/|
//  |     + \\
//  |     |  *
//  |     | /
//  +---->mux
//        |
//        out
module alu(op: i32, a: i32, b: i32) {
  _1 = (a + b)
  _2 = (a * b)
  _3 = select(op, _1, _2)
  // trigger next stage
}

````


````
module add(a: i32, b: i32) {
  _1 = a + b
  // trigger next stage
}
module mul(a: i32, b: i32) {
  _1 = a * b
  // trigger next stage
}
module alu(op, a: i32, b: i32) {
  trigger(add, [a, b]) when op == 0
  trigger(mul, [a, b]) when op == 1
}
````

