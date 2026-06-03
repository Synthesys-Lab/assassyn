"""Emit translation-validation artifacts."""

from __future__ import annotations

import json
from pathlib import Path

from .model import ValidationModel


def write_validation_json(model: ValidationModel, path: str | Path) -> None:
    """Write *model* as formatted translation-validation JSON."""

    Path(path).write_text(
        json.dumps(model.to_json_dict(), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_monitor_stub(model: ValidationModel, path: str | Path) -> None:
    """Write a bounded-simulation monitor stub for the current validation model."""

    lines = [
        "// Auto-generated translation-validation monitor scaffold.",
        "// The first implementation records the source-level mapping used by JSON.",
        "module translation_validation_monitor;",
        f"  localparam int ASSASSYN_VALIDATION_OBJECTS = {len(model.modules) + len(model.fifos)};",
        "endmodule",
        "",
    ]
    Path(path).write_text("\n".join(lines), encoding="utf-8")
