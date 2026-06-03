"""Emit translation-validation artifacts."""

from __future__ import annotations

import json
from pathlib import Path

from .model import ArrayTransition, FIFOTransition, ValidationModel


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

    ports = ["input logic clk", "input logic rst"]
    body = [
        "// Auto-generated translation-validation monitor.",
        "// Checks are intentionally local safety checks over RTL-visible schedule state.",
        "module translation_validation_monitor(",
    ]
    ports.extend(_trigger_ports(model))
    ports.extend(_fifo_ports(model))
    ports.extend(_array_ports(model))
    body.extend(_format_ports(ports))
    body.append(");")
    body.append("")
    body.append("  always_ff @(posedge clk) begin")
    body.append("    if (!rst) begin")
    body.extend(_trigger_assertions(model))
    body.extend(_fifo_assertions(model))
    body.extend(_array_assertions(model))
    body.append("    end")
    body.append("  end")
    body.append("endmodule")
    body.append("")
    body.append(_render_bind(model))
    body.append("")
    return "\n".join(body)


def _trigger_ports(model: ValidationModel) -> list[str]:
    """Return monitor ports for trigger-counter relations."""

    ports = []
    for trigger_id, trigger in model.triggers.items():
        prefix = _sv_identifier(trigger_id)
        width = _sv_width(trigger.width)
        ports.append(f"input logic {width}{prefix}_count")
        ports.append(f"input logic {width}{prefix}_delta")
    return ports


def _fifo_ports(model: ValidationModel) -> list[str]:
    """Return monitor ports for FIFO relations."""

    ports = []
    for fifo_id, fifo in model.fifos.items():
        prefix = _sv_identifier(fifo_id)
        count_width = _sv_width(fifo.rtl.count_width or 1)
        data_width = _sv_width(fifo.rtl.data_width or 1)
        ports.extend([
            f"input logic {count_width}{prefix}_count",
            f"input logic {prefix}_push_valid",
            f"input logic {prefix}_push_ready",
            f"input logic {prefix}_pop_valid",
            f"input logic {prefix}_pop_ready",
            f"input logic {data_width}{prefix}_pop_data",
        ])
    return ports


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


def _trigger_assertions(model: ValidationModel) -> list[str]:
    """Return monitor assertions for trigger counters."""

    lines: list[str] = []
    for trigger_id, trigger in model.triggers.items():
        prefix = _sv_identifier(trigger_id)
        max_depth = 1 << (int(trigger.width) - 1)
        lines.extend([
            _assert_known(f"{prefix}_count", trigger_id),
            _assert_known(f"{prefix}_delta", trigger_id),
            _assert_bounded(
                f"{prefix}_count",
                trigger.width,
                max_depth,
                trigger_id,
                "trigger count overflow",
            ),
        ])
    return lines


def _fifo_assertions(model: ValidationModel) -> list[str]:
    """Return monitor assertions for FIFO safety checks."""

    lines: list[str] = []
    for fifo_id, fifo in model.fifos.items():
        prefix = _sv_identifier(fifo_id)
        count_width = fifo.rtl.count_width or 1
        lines.extend([
            _assert_known(f"{prefix}_count", fifo_id),
            _assert_known(f"{prefix}_push_valid", fifo_id),
            _assert_known(f"{prefix}_push_ready", fifo_id),
            _assert_known(f"{prefix}_pop_valid", fifo_id),
            _assert_known(f"{prefix}_pop_ready", fifo_id),
            (
                f"      assert (!{prefix}_pop_valid || "
                f"!$isunknown({prefix}_pop_data)) "
                f"else $error(\"{_sv_string(fifo_id)} FIFO pop data unknown\");"
            ),
            _assert_bounded(
                f"{prefix}_count",
                count_width,
                fifo.configured_depth,
                fifo_id,
                "FIFO count overflow",
            ),
            (
                f"      assert (!({prefix}_push_valid && !{prefix}_push_ready)) "
                f"else $error(\"{_sv_string(fifo_id)} FIFO push without ready\");"
            ),
            (
                f"      assert (!({prefix}_pop_ready && !{prefix}_pop_valid)) "
                f"else $error(\"{_sv_string(fifo_id)} FIFO pop without valid\");"
            ),
        ])
    return lines


def _array_assertions(model: ValidationModel) -> list[str]:
    """Return monitor assertions for RegArray commit-boundary checks."""

    lines: list[str] = []
    for array_id, array in model.arrays.items():
        prefix = _sv_identifier(array_id)
        lines.extend(_array_write_assertions(prefix, array_id, array))
        lines.extend(_array_read_assertions(prefix, array_id, array))
        lines.extend(_array_visibility_assertions(prefix, array_id, array))
    return lines


def _array_write_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
) -> list[str]:
    """Return write-port X and next-cycle visibility assertions."""

    lines: list[str] = []
    for port in array.write_ports:
        port_prefix = f"{prefix}_w{port.port_index}"
        lines.extend([
            _assert_known(f"{port_prefix}_we", array_id),
            (
                f"      assert (!{port_prefix}_we || "
                f"!$isunknown({port_prefix}_widx)) "
                f"else $error(\"{_sv_string(array_id)} write index unknown\");"
            ),
            (
                f"      assert (!{port_prefix}_we || "
                f"!$isunknown({port_prefix}_wdata)) "
                f"else $error(\"{_sv_string(array_id)} write data unknown\");"
            ),
            (
                f"      assert (!($past(!rst) && $past({port_prefix}_we)) || "
                f"{port_prefix}_next_value == $past({port_prefix}_wdata)) "
                f"else $error(\"{_sv_string(array_id)} next-cycle payload mismatch\");"
            ),
        ])
    return lines


def _array_read_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
) -> list[str]:
    """Return read-port knownness assertions."""

    lines: list[str] = []
    for port in array.read_ports:
        port_prefix = f"{prefix}_r{port.port_index}"
        if port.read_index_signal is not None:
            lines.append(_assert_known(f"{port_prefix}_ridx", array_id))
        lines.append(_assert_known(f"{port_prefix}_rdata", array_id))
    return lines


def _array_visibility_assertions(
    prefix: str,
    array_id: str,
    array: ArrayTransition,
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
            lines.append(
                f"      assert (!({write_prefix}_we && {same_index}) || "
                f"{read_prefix}_rdata == {write_prefix}_next_value) "
                f"else $error(\"{_sv_string(array_id)} {message}\");"
            )
    return lines


def _render_bind(model: ValidationModel) -> str:
    """Return a bind statement that connects monitor ports to Top signals."""

    connections = [".clk(clk)", ".rst(rst)"]
    for trigger_id, trigger in model.triggers.items():
        prefix = _sv_identifier(trigger_id)
        connections.append(f".{prefix}_count({trigger.rtl_count_signal})")
        connections.append(f".{prefix}_delta({trigger.rtl_delta_signal})")
    for fifo_id, fifo in model.fifos.items():
        prefix = _sv_identifier(fifo_id)
        connections.extend(_fifo_bind_connections(prefix, fifo))
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


def _fifo_bind_connections(prefix: str, fifo: FIFOTransition) -> list[str]:
    """Return bind connections for one FIFO transition."""

    return [
        f".{prefix}_count({fifo.rtl.count_signal})",
        f".{prefix}_push_valid({fifo.rtl.push_valid_signal})",
        f".{prefix}_push_ready({fifo.rtl.ready_signal})",
        f".{prefix}_pop_valid({fifo.rtl.valid_signal})",
        f".{prefix}_pop_ready({fifo.rtl.pop_ready_signal})",
        f".{prefix}_pop_data({fifo.rtl.data_signal})",
    ]


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


def _assert_known(signal: str, source_id: str) -> str:
    """Return an assertion that rejects X/Z values."""

    return (
        f"      assert (!$isunknown({signal})) "
        f"else $error(\"{_sv_string(source_id)} {signal} unknown\");"
    )


def _assert_bounded(
    signal: str,
    width: int,
    bound: int,
    source_id: str,
    message: str,
) -> str:
    """Return an assertion that bounds an unsigned signal."""

    return (
        f"      assert ({signal} <= {_sv_literal(width, bound)}) "
        f"else $error(\"{_sv_string(source_id)} {message}\");"
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
