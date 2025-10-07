

### `PendingValue`

Data class used as bookkeeping for yet-to-be-named IR values.

#### Attributes
- `value: Any` – The IR node/value object

---

### `AssignmentTracker`

Queues freshly created values and provides name assignment when Python code
performs an assignment.

#### Methods

##### `__init__(self)`
Initializes an empty FIFO queue (`_pending_stack`), a `UniqueNameCache`, and
an `_enabled` switch that allows temporarily suspending tracking.

##### `push_value(self, value: Any)`
Adds a new `PendingValue` entry (unless tracking is disabled).

##### `clear_and_assign(self, assigned_name: str) -> List[Tuple[str, Any]]`
When a Python assignment happens, this method drains the queue in creation
order, generates names, applies them to the tracked objects, and returns the
list of `(name, value)` pairs.

Name selection prefers the explicit `assigned_name`, and otherwise uses
`_get_type_prefix`. All names are
made unique through the shared `UniqueNameCache`. The tracker attempts to set
`_name` or `name` on the value (and silently ignores failures) so the IR
object reflects the generated identifier.

##### `_generate_name(self, value: Any, assigned_name: str) -> str`
Helper that chooses the base name and feeds it to the unique-name cache.

##### `_get_type_prefix(self, value: Any) -> str`
Placeholder implementation that currently returns `"val"`. Specialised namers
override this behaviour.

##### `disable(self)` / `enable(self)`
Toggle tracking on and off. While disabled, calls to `push_value` and
`clear_and_assign` become no-ops.
