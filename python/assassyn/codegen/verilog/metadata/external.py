"""External module metadata helpers for Verilog code generation."""

from __future__ import annotations

from dataclasses import dataclass
from types import MappingProxyType
from typing import Dict, List, Mapping, Tuple, TYPE_CHECKING, cast

if TYPE_CHECKING:
    from ....ir.module import Module
    from ....ir.expr.intrinsic import ExternalIntrinsic, PureIntrinsic
    from ....ir.value import Value
else:  # pragma: no cover - runtime imports for type checking only
    from ....ir.module import Module  # type: ignore
    from ....ir.expr.intrinsic import ExternalIntrinsic, PureIntrinsic  # type: ignore
    from ....ir.value import Value  # type: ignore


@dataclass(frozen=True)
class ExternalRead:
    """Cross-module external output read recorded during analysis."""

    expr: "PureIntrinsic"
    producer: "Module"
    consumer: "Module"
    instance: "ExternalIntrinsic"
    port_name: str
    index_operand: "Value | None"


class ExternalRegistry:
    """Accumulated external module metadata shared across Verilog emission."""

    def __init__(self) -> None:
        self._classes: List[type] | Tuple[type, ...] = []
        self._instance_owners: (
            Dict["ExternalIntrinsic", "Module"]
            | Mapping["ExternalIntrinsic", "Module"]
        ) = {}
        self._reads: List[ExternalRead] | Tuple[ExternalRead, ...] = []
        self._reads_by_consumer: (
            Dict["Module", List[ExternalRead]]
            | Mapping["Module", Tuple[ExternalRead, ...]]
        ) = {}
        self._reads_by_instance: (
            Dict["ExternalIntrinsic", List[ExternalRead]]
            | Mapping["ExternalIntrinsic", Tuple[ExternalRead, ...]]
        ) = {}
        self._reads_by_producer: (
            Dict["Module", List[ExternalRead]]
            | Mapping["Module", Tuple[ExternalRead, ...]]
        ) = {}
        self._frozen = False

    # --------------------------------------------------------------------- #
    # Recording helpers
    # --------------------------------------------------------------------- #
    def record_class(self, ext_class: type) -> None:
        """Track an external class encountered during analysis."""
        self._ensure_mutable()
        if ext_class not in self._classes:
            self._classes.append(ext_class)

    def record_instance(self, instance: "ExternalIntrinsic", owner: "Module") -> None:
        """Associate an external instance with its owning module."""
        self._ensure_mutable()
        self._instance_owners[instance] = owner
        self.record_class(instance.external_class)

    def record_cross_module_read(self, record: ExternalRead) -> None:
        """Register a cross-module read of an external output."""
        self._ensure_mutable()
        self._reads.append(record)
        self._reads_by_consumer.setdefault(record.consumer, []).append(record)
        self._reads_by_instance.setdefault(record.instance, []).append(record)
        self._reads_by_producer.setdefault(record.producer, []).append(record)

    # --------------------------------------------------------------------- #
    # Accessors
    # --------------------------------------------------------------------- #
    @property
    def classes(self) -> Tuple[type, ...]:
        """Return the external classes discovered during analysis."""
        if not self._frozen:
            return tuple(self._classes)  # type: ignore[arg-type]
        return cast(Tuple[type, ...], self._classes)

    @property
    def instance_owners(self) -> Mapping["ExternalIntrinsic", "Module"]:
        """Expose the mapping from external instances to owner modules."""
        if not self._frozen:
            return MappingProxyType(self._instance_owners)  # type: ignore[arg-type]
        return cast(Mapping[ExternalIntrinsic, Module], self._instance_owners)

    @property
    def cross_module_reads(self) -> Tuple[ExternalRead, ...]:
        """Return all recorded cross-module external output reads."""
        if not self._frozen:
            return tuple(self._reads)  # type: ignore[arg-type]
        return cast(Tuple[ExternalRead, ...], self._reads)

    def reads_for_consumer(self, module: "Module") -> Tuple[ExternalRead, ...]:
        """Return reads where *module* consumes an external output."""
        records = self._reads_by_consumer.get(module, ())
        if not self._frozen:
            return tuple(records)  # type: ignore[arg-type]
        return cast(Tuple[ExternalRead, ...], records)

    def reads_for_instance(self, instance: "ExternalIntrinsic") -> Tuple[ExternalRead, ...]:
        """Return reads targeting *instance*."""
        records = self._reads_by_instance.get(instance, ())
        if not self._frozen:
            return tuple(records)  # type: ignore[arg-type]
        return cast(Tuple[ExternalRead, ...], records)

    def reads_for_producer(self, module: "Module") -> Tuple[ExternalRead, ...]:
        """Return reads pulling values from external instances owned by *module*."""
        records = self._reads_by_producer.get(module, ())
        if not self._frozen:
            return tuple(records)  # type: ignore[arg-type]
        return cast(Tuple[ExternalRead, ...], records)

    def owner_for(self, instance: "ExternalIntrinsic") -> "Module | None":
        """Return the owning module for *instance*."""
        owners = self._instance_owners
        if not self._frozen:
            return owners.get(instance)
        return cast(Mapping[ExternalIntrinsic, Module], owners).get(instance)

    # --------------------------------------------------------------------- #
    # Freezing
    # --------------------------------------------------------------------- #
    def freeze(self) -> None:
        """Convert all internal stores into immutable views."""
        if self._frozen:
            return

        self._classes = cast(Tuple[type, ...], tuple(self._classes))
        self._instance_owners = cast(
            Mapping[ExternalIntrinsic, Module],
            MappingProxyType(dict(self._instance_owners)),
        )
        self._reads = cast(Tuple[ExternalRead, ...], tuple(self._reads))
        self._reads_by_consumer = cast(
            Mapping[Module, Tuple[ExternalRead, ...]],
            MappingProxyType(
                {module: tuple(records) for module, records in self._reads_by_consumer.items()}
            ),
        )
        self._reads_by_instance = cast(
            Mapping[ExternalIntrinsic, Tuple[ExternalRead, ...]],
            MappingProxyType(
                {instance: tuple(records) for instance, records in self._reads_by_instance.items()}
            ),
        )
        self._reads_by_producer = cast(
            Mapping[Module, Tuple[ExternalRead, ...]],
            MappingProxyType(
                {module: tuple(records) for module, records in self._reads_by_producer.items()}
            ),
        )
        self._frozen = True

    @property
    def frozen(self) -> bool:
        """Return whether the registry has been frozen."""
        return self._frozen

    def _ensure_mutable(self) -> None:
        if self._frozen:
            raise RuntimeError("ExternalRegistry is frozen; cannot record new entries")


__all__ = [
    "ExternalRead",
    "ExternalRegistry",
]
