# The Way to New Namify
- [ ] Simplify the test cases of unique name cache.
- [ ] An assignment rewriter that passes all the test cases.
- [ ] Implement the assignment rewriter by pushing values to builder's stack.
- [ ] On an assignment on Assassyn value, clear the builder's stack, and assign names to all the values in the stack.
    - [ ] We need a type oriented namer for each type. DO NOT make this a method of each type. Make this a centralized function.
- [ ] Hook this new namify into our combinational_for decorator.
