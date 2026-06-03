"""Dataclasses for normalized translation-validation models."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

VALIDATION_SCHEMA = "assassyn.translation_validation.v1"


@dataclass(frozen=True)
class RTLSignalMap:  # pylint: disable=too-many-instance-attributes
    """RTL signal names corresponding to one semantic object."""

    count_signal: str | None = None
    count_width: int | None = None
    push_valid_signal: str | None = None
    valid_signal: str | None = None
    ready_signal: str | None = None
    pop_ready_signal: str | None = None
    data_signal: str | None = None
    data_width: int | None = None


@dataclass(frozen=True)
class ModuleTransition:
    """Normalized module fire equation metadata."""

    coverage_id: str
    module: str
    fire_signal: str
    event_count_signal: str | None = None


@dataclass(frozen=True)
class FIFOTransition:
    """Normalized FIFO queue relation metadata."""

    coverage_id: str
    module: str
    port: str
    configured_depth_log2: int
    configured_depth: int
    rtl: RTLSignalMap


@dataclass(frozen=True)
class TriggerTransition:
    """Normalized trigger-counter relation metadata."""

    coverage_id: str
    module: str
    rtl_count_signal: str
    rtl_delta_signal: str
    width: int


@dataclass(frozen=True)
class AsyncCallTransition:
    """Normalized async-call relation metadata."""

    coverage_id: str
    caller: str
    callee: str
    fifo_ids: tuple[str, ...]


@dataclass
class ValidationModel:
    """Container for all normalized validation relations."""

    schema: str = VALIDATION_SCHEMA
    modules: dict[str, ModuleTransition] = field(default_factory=dict)
    fifos: dict[str, FIFOTransition] = field(default_factory=dict)
    triggers: dict[str, TriggerTransition] = field(default_factory=dict)
    async_calls: dict[str, AsyncCallTransition] = field(default_factory=dict)

    def to_json_dict(self) -> dict[str, Any]:
        """Return a JSON-serializable representation of the model."""

        return {
            "schema": self.schema,
            "modules": {
                key: vars(value)
                for key, value in self.modules.items()
            },
            "fifos": {
                key: _fifo_to_dict(value)
                for key, value in self.fifos.items()
            },
            "triggers": {
                key: vars(value)
                for key, value in self.triggers.items()
            },
            "async_calls": {
                key: vars(value)
                for key, value in self.async_calls.items()
            },
        }


def _fifo_to_dict(value: FIFOTransition) -> dict[str, Any]:
    """Convert a FIFO transition to JSON data."""

    data = vars(value).copy()
    data["rtl"] = vars(value.rtl)
    return data
