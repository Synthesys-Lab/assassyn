```python
# The metadata implementation now lives in python.assassyn.codegen.verilog.metadata.[core|module|array|fifo|external]

class InteractionMatrix:
    modules: dict[Module, ModuleBucket]
    arrays: dict[Array, ArrayBucket]
    fifos: dict[Port, FIFOBucket]
    async_ledger: AsyncLedger
    finish_sites: dict[Module, list[Intrinsic]]

class ModuleBucket:
    array_reads: dict[Array, list[ArrayRead]]
    array_writes: dict[Array, list[ArrayWrite]]
    fifo_ports: dict[Port, list[FIFOExpr]]
    pushes: list[FIFOPush]
    pops: list[FIFOPop]

class ArrayBucket:
    reads: list[ArrayRead]
    writers: dict[Module, list[ArrayWrite]]

class FIFOBucket:
    pushes: list[FIFOPush]
    pops: list[FIFOPop]

matrix = InteractionMatrix()

for mod in sys.modules[:] + sys.downstreams[:]:
    matrix.modules.setdefault(mod, ModuleBucket())

for port in sys.fifo_ports:
    matrix.fifos.setdefault(port, FIFOBucket())

for arr in sys.arrays:
    matrix.arrays.setdefault(arr, ArrayBucket())

for mod in sys.modules[:] + sys.downstreams[:]:
    for expr in mod.body:
        if isinstance(expr, ArrayWrite):
            matrix.record(module=mod, resource=expr.array, kind=ARRAY_WRITE, expr=expr)
        elif isinstance(expr, ArrayRead):
            matrix.record(module=mod, resource=expr.array, kind=ARRAY_READ, expr=expr)
        elif isinstance(expr, FIFOPush):
            matrix.record(module=mod, resource=expr.fifo, kind=FIFO_PUSH, expr=expr)
        elif isinstance(expr, FIFOPop):
            matrix.record(module=mod, resource=expr.fifo, kind=FIFO_POP, expr=expr)
        elif isinstance(expr, AsyncCall):
            matrix.async_ledger.record(module=mod, callee=expr.callee, call=expr)
        elif isinstance(expr, Intrinsic) and expr.opcode == Intrinsic.FINISH:
            matrix.finish_sites.setdefault(mod, []).append(expr)
```
