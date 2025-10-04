# Experimental Frontend

This document explains the experimental frontend of Assassyn.
The old frontend is self-examplified in continous integration tests
in [ci-tests](../../../ci-tests/). The experimental frontend aims
at providing a more functional programming flavor to users.

## Features

### Module Factory

The key features of the new frontend include constructing 3 types of modules:
- Pipeline Stage: The most common type of pipeline stages;
- Convergent Downstream: To which combinational logics converges;

Refer to [module.md](./module.md), and [downstream.md](./downstream.md) for more details.

>RFC: Do we want a `Callback` module type?

## Implementation

- `factory.{md/py}` describes and implements the common interface of
  instantiating different kinds of modules.
- `module.{md/py}` describes and implements `Module` type, which is
  the most common type of pipeline stages.
- `downstream.{md/py}` describes and implements `Downstream` type,
  which converges combinational logics across different stages or downstreams.