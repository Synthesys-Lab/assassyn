"""Emit translation-validation artifacts."""

from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path

from .model import ArrayTransition, ValidationModel


def write_validation_json(model: ValidationModel, path: str | Path) -> None:
    """Write *model* as formatted translation-validation JSON."""

    Path(path).write_text(
        json.dumps(model.to_json_dict(), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_monitor_stub(model: ValidationModel, path: str | Path) -> None:
    """Write a bounded-simulation monitor for the current validation model."""

    Path(path).write_text(render_monitor(model), encoding="utf-8")


def render_monitor(model: ValidationModel) -> str:
    """Render a SystemVerilog bounded-simulation validation monitor."""

    assertions = _AssertionCounterBuilder()
    ports = ["input logic clk", "input logic rst"]
    body = [
        "// Auto-generated translation-validation monitor.",
        "// Checks are intentionally local safety checks over RTL-visible schedule state.",
        "module translation_validation_monitor(",
    ]
    ports.extend(_array_ports(model))
    body.extend(_format_ports(ports))
    body.append(");")
    body.append("")
    assertion_lines = [
        *_array_assertions(model, assertions),
    ]
    body.extend(assertions.declarations())
    body.append("  always_ff @(posedge clk) begin")
    body.append("    if (!rst) begin")
    body.extend(assertion_lines)
    body.append("    end")
    body.append("  end")
    body.extend(assertions.final_report())
    body.append("endmodule")
    body.append("")
    body.append(_render_bind(model))
    if model.fifos:
        body.append("")
        body.append(_render_fifo_monitor())
    if model.triggers:
        body.append("")
        body.append(_render_trigger_counter_monitor())
    body.append("")
    return "\n".join(body)


def _array_ports(model: ValidationModel) -> list[str]:
    """Return monitor ports for RegArray relations."""

    ports = []
    for array_id, array in model.arrays.items():
        prefix = _sv_identifier(array_id)
        index_width = _sv_width(array.index_width)
        data_width = _sv_width(array.data_width)
        for port in array.write_ports:
            port_prefix = f"{prefix}_w{port.port_index}"
            ports.extend([
                f"input logic {port_prefix}_we",
                f"input logic {index_width}{port_prefix}_widx",
                f"input logic {data_width}{port_prefix}_wdata",
                f"input logic {data_width}{port_prefix}_next_value",
            ])
        for port in array.read_ports:
            port_prefix = f"{prefix}_r{port.port_index}"
            if port.read_index_signal is not None:
                ports.append(f"input logic {index_width}{port_prefix}_ridx")
            ports.append(f"input logic {data_width}{port_prefix}_rdata")
    return ports


@dataclass(frozen=True)
class _AssertionCounter:
    """One emitted monitor assertion counter."""

    name: str
    activation_counter: str
    failure_counter: str


class _AssertionCounterBuilder:
    """Build assertion statements and final counter reports."""

    def __init__(self):
        self._counters: list[_AssertionCounter] = []

    def assertion(
        self,
        *,
        name: str,
        activation: str,
        condition: str,
        message: str,
    ) -> list[str]:
        """Return assertion lines with activation and failure counters."""

        index = len(self._counters)
        prefix = f"tv_assert_{index}"
        activation_counter = f"{prefix}_activations"
        failure_counter = f"{prefix}_failures"
        self._counters.append(
            _AssertionCounter(
                name=name,
                activation_counter=activation_counter,
                failure_counter=failure_counter,
            )
        )
        return [
            f"      if ({activation}) begin",
            f"        {activation_counter} <= {activation_counter} + 64'd1;",
            "      end",
            f"      assert (!({activation}) || ({condition}))",
            "      else begin",
            f"        {failure_counter} <= {failure_counter} + 64'd1;",
            f"        $error(\"{_sv_string(message)}\");",
            "      end",
        ]

    def declarations(self) -> list[str]:
        """Return SystemVerilog counter declarations."""

        lines: list[str] = []
        for counter in self._counters:
            lines.append(f"  longint unsigned {counter.activation_counter} = 0;")
            lines.append(f"  longint unsigned {counter.failure_counter} = 0;")
        if lines:
            lines.append("")
        return lines

    def final_report(self) -> list[str]:
        """Return final `$display` lines for all counters."""

        if not self._counters:
            return []

        lines = ["", "  final begin"]
        for counter in self._counters:
            lines.append(
                "    $display(\"translation_validation_assertion "
                f"name={_sv_string(counter.name)} "
                "activations=%0d failures=%0d\", "
                f"{counter.activation_counter}, {counter.failure_counter});"
            )
        lines.append("  end")
        return lines


def _array_assertions(
    model: ValidationModel,
    assertions: _AssertionCounterBuilder,
) -> list[str]:
    """Return monitor assertions for RegArray commit-boundary checks."""

    lines: list[str] = []
    for array_id, array in model.arrays.items():
        prefix = _sv_identifier(array_id)
        lines.extend(_array_write_assertions(prefix, array_id, array, assertions))
        lines.extend(_array_read_assertions(prefix, array_id, array, assertions))
        lines.extend(_array_visibility_assertions(prefix, array_id, array, assertions))
    return lines


def _array_write_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
    assertions: _AssertionCounterBuilder,
) -> list[str]:
    """Return write-port X and next-cycle visibility assertions."""

    lines: list[str] = []
    for port in array.write_ports:
        port_prefix = f"{prefix}_w{port.port_index}"
        lines.extend(
            _assert_known(
                assertions,
                name=f"{array_id}.w{port.port_index}.enable_known",
                signal=f"{port_prefix}_we",
                source_id=array_id,
            )
        )
        lines.extend(
            assertions.assertion(
                name=f"{array_id}.w{port.port_index}.index_known",
                activation=f"{port_prefix}_we",
                condition=f"!$isunknown({port_prefix}_widx)",
                message=f"{array_id} write index unknown",
            )
        )
        lines.extend(
            assertions.assertion(
                name=f"{array_id}.w{port.port_index}.data_known",
                activation=f"{port_prefix}_we",
                condition=f"!$isunknown({port_prefix}_wdata)",
                message=f"{array_id} write data unknown",
            )
        )
        lines.extend(
            assertions.assertion(
                name=f"{array_id}.w{port.port_index}.next_cycle_payload",
                activation=f"$past(!rst) && $past({port_prefix}_we)",
                condition=f"{port_prefix}_next_value == $past({port_prefix}_wdata)",
                message=f"{array_id} next-cycle payload mismatch",
            )
        )
    return lines


def _array_read_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
    assertions: _AssertionCounterBuilder,
) -> list[str]:
    """Return read-port knownness assertions."""

    lines: list[str] = []
    for port in array.read_ports:
        port_prefix = f"{prefix}_r{port.port_index}"
        if port.read_index_signal is not None:
            lines.extend(
                _assert_known(
                    assertions,
                    name=f"{array_id}.r{port.port_index}.index_known",
                    signal=f"{port_prefix}_ridx",
                    source_id=array_id,
                )
            )
        lines.extend(
            _assert_known(
                assertions,
                name=f"{array_id}.r{port.port_index}.data_known",
                signal=f"{port_prefix}_rdata",
                source_id=array_id,
            )
        )
    return lines


def _array_visibility_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
    assertions: _AssertionCounterBuilder,
) -> list[str]:
    """Return no-same-cycle-visibility assertions for read/write aliases."""

    lines: list[str] = []
    message = "same-cycle write visible through read port"
    for write in array.write_ports:
        write_prefix = f"{prefix}_w{write.port_index}"
        for read in array.read_ports:
            read_prefix = f"{prefix}_r{read.port_index}"
            same_index = (
                f"{read_prefix}_ridx == {write_prefix}_widx"
                if read.read_index_signal is not None
                else "1'b1"
            )
            lines.extend(
                assertions.assertion(
                    name=(
                        f"{array_id}.w{write.port_index}.r{read.port_index}."
                        "same_cycle_visibility"
                    ),
                    activation=f"{write_prefix}_we && {same_index}",
                    condition=f"{read_prefix}_rdata == {write_prefix}_next_value",
                    message=f"{array_id} {message}",
                )
            )
    return lines


def _render_bind(model: ValidationModel) -> str:
    """Return a bind statement that connects monitor ports to Top signals."""

    connections = [".clk(clk)", ".rst(rst)"]
    for array_id, array in model.arrays.items():
        prefix = _sv_identifier(array_id)
        connections.extend(_array_bind_connections(prefix, array))

    joined = ",\n    ".join(connections)
    return (
        "bind Top translation_validation_monitor "
        "translation_validation_monitor_inst(\n"
        f"    {joined}\n"
        ");"
    )


def _render_fifo_monitor() -> str:
    """Return a type bind for generated FIFO safety checks."""

    return "\n".join([
        "module translation_validation_fifo_monitor #(",
        "  parameter longint WIDTH = 1,",
        "  parameter longint DEPTH_LOG2 = 1",
        ")(",
        "  input logic clk,",
        "  input logic rst_n,",
        "  input logic push_valid,",
        "  input logic [WIDTH-1:0] push_data,",
        "  input logic push_ready,",
        "  input logic pop_valid,",
        "  input logic [WIDTH-1:0] pop_data,",
        "  input logic pop_ready",
        ");",
        "  longint unsigned tv_fifo_push_valid_known_activations = 0;",
        "  longint unsigned tv_fifo_push_valid_known_failures = 0;",
        "  longint unsigned tv_fifo_push_ready_known_activations = 0;",
        "  longint unsigned tv_fifo_push_ready_known_failures = 0;",
        "  longint unsigned tv_fifo_pop_valid_known_activations = 0;",
        "  longint unsigned tv_fifo_pop_valid_known_failures = 0;",
        "  longint unsigned tv_fifo_pop_ready_known_activations = 0;",
        "  longint unsigned tv_fifo_pop_ready_known_failures = 0;",
        "  longint unsigned tv_fifo_pop_data_known_activations = 0;",
        "  longint unsigned tv_fifo_pop_data_known_failures = 0;",
        "",
        "  always_ff @(posedge clk) begin",
        "    if (rst_n) begin",
        (
            "      tv_fifo_push_valid_known_activations <= "
            "tv_fifo_push_valid_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(push_valid))",
        "      else begin",
        (
            "        tv_fifo_push_valid_known_failures <= "
            "tv_fifo_push_valid_known_failures + 64'd1;"
        ),
        "        $error(\"fifo push_valid unknown\");",
        "      end",
        (
            "      tv_fifo_push_ready_known_activations <= "
            "tv_fifo_push_ready_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(push_ready))",
        "      else begin",
        (
            "        tv_fifo_push_ready_known_failures <= "
            "tv_fifo_push_ready_known_failures + 64'd1;"
        ),
        "        $error(\"fifo push_ready unknown\");",
        "      end",
        (
            "      tv_fifo_pop_valid_known_activations <= "
            "tv_fifo_pop_valid_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(pop_valid))",
        "      else begin",
        (
            "        tv_fifo_pop_valid_known_failures <= "
            "tv_fifo_pop_valid_known_failures + 64'd1;"
        ),
        "        $error(\"fifo pop_valid unknown\");",
        "      end",
        (
            "      tv_fifo_pop_ready_known_activations <= "
            "tv_fifo_pop_ready_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(pop_ready))",
        "      else begin",
        (
            "        tv_fifo_pop_ready_known_failures <= "
            "tv_fifo_pop_ready_known_failures + 64'd1;"
        ),
        "        $error(\"fifo pop_ready unknown\");",
        "      end",
        "      if (pop_valid) begin",
        (
            "        tv_fifo_pop_data_known_activations <= "
            "tv_fifo_pop_data_known_activations + 64'd1;"
        ),
        "      end",
        "      assert (!pop_valid || !$isunknown(pop_data))",
        "      else begin",
        (
            "        tv_fifo_pop_data_known_failures <= "
            "tv_fifo_pop_data_known_failures + 64'd1;"
        ),
        "        $error(\"fifo pop_data unknown\");",
        "      end",
        "    end",
        "  end",
        "",
        "  final begin",
        (
            "    $display(\"translation_validation_assertion "
            "name=fifo.push_valid_known activations=%0d failures=%0d\", "
            "tv_fifo_push_valid_known_activations, "
            "tv_fifo_push_valid_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=fifo.push_ready_known activations=%0d failures=%0d\", "
            "tv_fifo_push_ready_known_activations, "
            "tv_fifo_push_ready_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=fifo.pop_valid_known activations=%0d failures=%0d\", "
            "tv_fifo_pop_valid_known_activations, "
            "tv_fifo_pop_valid_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=fifo.pop_ready_known activations=%0d failures=%0d\", "
            "tv_fifo_pop_ready_known_activations, "
            "tv_fifo_pop_ready_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=fifo.pop_data_known activations=%0d failures=%0d\", "
            "tv_fifo_pop_data_known_activations, "
            "tv_fifo_pop_data_known_failures);"
        ),
        "  end",
        "endmodule",
        "",
        "bind fifo translation_validation_fifo_monitor #(",
        "  .WIDTH(WIDTH),",
        "  .DEPTH_LOG2(DEPTH_LOG2)",
        ") translation_validation_fifo_monitor_inst (",
        "  .clk(clk),",
        "  .rst_n(rst_n),",
        "  .push_valid(push_valid),",
        "  .push_data(push_data),",
        "  .push_ready(push_ready),",
        "  .pop_valid(pop_valid),",
        "  .pop_data(pop_data),",
        "  .pop_ready(pop_ready)",
        ");",
    ])


def _render_trigger_counter_monitor() -> str:
    """Return a type bind for trigger-counter safety checks."""

    return "\n".join([
        "module translation_validation_trigger_monitor #(",
        "  parameter longint WIDTH = 1",
        ")(",
        "  input logic clk,",
        "  input logic rst_n,",
        "  input logic [WIDTH-1:0] count,",
        "  input logic [WIDTH-1:0] delta",
        ");",
        "  localparam logic [WIDTH-1:0] MAX_DEPTH = (1 << (WIDTH - 1));",
        "  longint unsigned tv_trigger_count_known_activations = 0;",
        "  longint unsigned tv_trigger_count_known_failures = 0;",
        "  longint unsigned tv_trigger_delta_known_activations = 0;",
        "  longint unsigned tv_trigger_delta_known_failures = 0;",
        "  longint unsigned tv_trigger_count_bounded_activations = 0;",
        "  longint unsigned tv_trigger_count_bounded_failures = 0;",
        "",
        "  always_ff @(posedge clk) begin",
        "    if (rst_n) begin",
        (
            "      tv_trigger_count_known_activations <= "
            "tv_trigger_count_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(count))",
        "      else begin",
        (
            "        tv_trigger_count_known_failures <= "
            "tv_trigger_count_known_failures + 64'd1;"
        ),
        "        $error(\"trigger_counter count unknown\");",
        "      end",
        (
            "      tv_trigger_delta_known_activations <= "
            "tv_trigger_delta_known_activations + 64'd1;"
        ),
        "      assert (!$isunknown(delta))",
        "      else begin",
        (
            "        tv_trigger_delta_known_failures <= "
            "tv_trigger_delta_known_failures + 64'd1;"
        ),
        "        $error(\"trigger_counter delta unknown\");",
        "      end",
        (
            "      tv_trigger_count_bounded_activations <= "
            "tv_trigger_count_bounded_activations + 64'd1;"
        ),
        "      assert (count <= MAX_DEPTH)",
        "      else begin",
        (
            "        tv_trigger_count_bounded_failures <= "
            "tv_trigger_count_bounded_failures + 64'd1;"
        ),
        "        $error(\"trigger_counter trigger count overflow\");",
        "      end",
        "    end",
        "  end",
        "",
        "  final begin",
        (
            "    $display(\"translation_validation_assertion "
            "name=trigger_counter.count_known activations=%0d failures=%0d\", "
            "tv_trigger_count_known_activations, "
            "tv_trigger_count_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=trigger_counter.delta_known activations=%0d failures=%0d\", "
            "tv_trigger_delta_known_activations, "
            "tv_trigger_delta_known_failures);"
        ),
        (
            "    $display(\"translation_validation_assertion "
            "name=trigger_counter.count_bounded activations=%0d failures=%0d\", "
            "tv_trigger_count_bounded_activations, "
            "tv_trigger_count_bounded_failures);"
        ),
        "  end",
        "endmodule",
        "",
        "bind trigger_counter translation_validation_trigger_monitor #(",
        "  .WIDTH(WIDTH)",
        ") translation_validation_trigger_monitor_inst (",
        "  .clk(clk),",
        "  .rst_n(rst_n),",
        "  .count(count),",
        "  .delta(delta)",
        ");",
    ])


def _array_bind_connections(prefix: str, array: ArrayTransition) -> list[str]:
    """Return bind connections for one RegArray transition."""

    connections: list[str] = []
    for port in array.write_ports:
        port_prefix = f"{prefix}_w{port.port_index}"
        connections.extend([
            f".{port_prefix}_we({port.write_enable_signal})",
            f".{port_prefix}_widx({port.write_index_signal})",
            f".{port_prefix}_wdata({port.write_data_signal})",
            f".{port_prefix}_next_value({port.next_value_signal})",
        ])
    for port in array.read_ports:
        port_prefix = f"{prefix}_r{port.port_index}"
        if port.read_index_signal is not None:
            connections.append(f".{port_prefix}_ridx({port.read_index_signal})")
        connections.append(f".{port_prefix}_rdata({port.read_data_signal})")
    return connections


def _format_ports(ports: list[str]) -> list[str]:
    """Format a SystemVerilog port list."""

    lines = []
    for index, port in enumerate(ports):
        suffix = "," if index != len(ports) - 1 else ""
        lines.append(f"  {port}{suffix}")
    return lines


def _assert_known(
    assertions: _AssertionCounterBuilder,
    *,
    name: str,
    signal: str,
    source_id: str,
) -> list[str]:
    """Return an assertion that rejects X/Z values."""

    return assertions.assertion(
        name=name,
        activation="1'b1",
        condition=f"!$isunknown({signal})",
        message=f"{source_id} {signal} unknown",
    )


def _assert_bounded(
    assertions: _AssertionCounterBuilder,
    *,
    name: str,
    signal: str,
    width: int,
    bound: int,
    source_id: str,
    message: str,
) -> list[str]:
    """Return an assertion that bounds an unsigned signal."""

    return assertions.assertion(
        name=name,
        activation="1'b1",
        condition=f"{signal} <= {_sv_literal(width, bound)}",
        message=f"{source_id} {message}",
    )


def _sv_width(width: int) -> str:
    """Return a SystemVerilog packed width prefix."""

    if int(width) <= 1:
        return ""
    return f"[{int(width) - 1}:0] "


def _sv_literal(width: int, value: int) -> str:
    """Return an unsigned SystemVerilog literal with a fixed width."""

    return f"{max(1, int(width))}'d{int(value)}"


def _sv_identifier(value: str) -> str:
    """Return a stable SystemVerilog identifier fragment."""

    return "".join(ch if ch.isalnum() else "_" for ch in value)


def _sv_string(value: str) -> str:
    """Escape a string for SystemVerilog diagnostics."""

    return value.replace("\\", "\\\\").replace('"', '\\"')
