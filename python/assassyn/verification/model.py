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


@dataclass(frozen=True)
class ArrayWritePortTransition:
    """Normalized RegArray write-port commit-boundary metadata."""

    writer: str
    port_index: int
    write_enable_signal: str
    write_index_signal: str
    write_data_signal: str
    next_value_signal: str


@dataclass(frozen=True)
class ArrayReadPortTransition:
    """Normalized RegArray read-port metadata."""

    reader: str
    port_index: int
    read_index_signal: str | None
    read_data_signal: str


@dataclass(frozen=True)
class ArrayTransition:
    """Normalized RegArray read/write relation metadata."""

    coverage_id: str
    array: str
    depth: int
    index_width: int
    data_width: int
    write_ports: tuple[ArrayWritePortTransition, ...]
    read_ports: tuple[ArrayReadPortTransition, ...] = ()


@dataclass
class ValidationModel:
    """Container for all normalized validation relations."""

    schema: str = VALIDATION_SCHEMA
    modules: dict[str, ModuleTransition] = field(default_factory=dict)
    fifos: dict[str, FIFOTransition] = field(default_factory=dict)
    triggers: dict[str, TriggerTransition] = field(default_factory=dict)
    async_calls: dict[str, AsyncCallTransition] = field(default_factory=dict)
    arrays: dict[str, ArrayTransition] = field(default_factory=dict)

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
            "arrays": {
                key: _array_to_dict(value)
                for key, value in self.arrays.items()
            },
        }


def _fifo_to_dict(value: FIFOTransition) -> dict[str, Any]:
    """Convert a FIFO transition to JSON data."""

    data = vars(value).copy()
    data["rtl"] = vars(value.rtl)
    return data


def _array_to_dict(value: ArrayTransition) -> dict[str, Any]:
    """Convert an array transition to JSON data."""

    data = vars(value).copy()
    data["write_ports"] = [vars(port) for port in value.write_ports]
    data["read_ports"] = [vars(port) for port in value.read_ports]
    return data
