# Architecture Template of Assassyn

## Credit-based Pipeline Stages

As shown below, assassyn generates architectures with credit-based architecture pipeline
stages. When the architecture has a pending activation, it gains a credit. Once it is
succefully activated, it consumes the credit. The architecture accepts data from the
FIFOs from prior stages.

````
    +-------+
--->|credit |-------+
    +-------+       |--[O1]->
-[I1]->|    Stage   |--[O2]->
-[I2]->|            |
       +------------+
[A] [B] are stage registers.

````

## Cross-stage Combinational Communication

As shown below, credited pipeline stages can also have combinational pins
that connect to downstream modules. And downstream modules can recursively
connect to other downstream modules, or other pipeline stages.

````
    +--------+
    | credit |-------+
    +--------+       |             +--------------+
       |    Stage1   |------------>|              |---> Other modules
       |             |             | Downstream   |
       +-------------+             |              |
                                   +--------------+
    +--------+                          ^
    | credit |-------+                  |
    +--------+       |------------------+
       |    Stage2   |
       |             |
       +-------------+
````
