"""Emit translation-validation artifacts."""

from __future__ import annotations

import json
from pathlib import Path

from .model import FIFOTransition, ValidationModel


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
    body.extend(_format_ports(ports))
    body.append(");")
    body.append("")
    body.append("  always_ff @(posedge clk) begin")
    body.append("    if (!rst) begin")
    body.extend(_trigger_assertions(model))
    body.extend(_fifo_assertions(model))
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


def _trigger_assertions(model: ValidationModel) -> list[str]:
    """Return monitor assertions for trigger counters."""

    lines: list[str] = []
    for trigger_id, trigger in model.triggers.items():
        prefix = _sv_identifier(trigger_id)
        max_depth = 1 << (int(trigger.width) - 1)
        lines.extend([
            _assert_known(f"{prefix}_count", trigger_id),
            _assert_known(f"{prefix}_delta", trigger_id),
            (
                f"      assert ({prefix}_count <= {_sv_literal(trigger.width, max_depth)}) "
                f"else $error(\"{_sv_string(trigger_id)} trigger count overflow\");"
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
            (
                f"      assert ({prefix}_count <= "
                f"{_sv_literal(count_width, fifo.configured_depth)}) "
                f"else $error(\"{_sv_string(fifo_id)} FIFO count overflow\");"
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
