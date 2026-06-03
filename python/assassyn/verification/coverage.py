"""Semantic coverage JSON helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping, MutableMapping

COVERAGE_SCHEMA = "assassyn.semantic_coverage.v1"


def load_coverage(path: str | Path) -> dict[str, Any]:
    """Load semantic coverage JSON from *path*."""

    with Path(path).open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError("Coverage artifact must be a JSON object")
    return data


def validate_coverage_schema(coverage: Mapping[str, Any]) -> None:
    """Validate required fields in a semantic coverage artifact."""

    if coverage.get("schema") != COVERAGE_SCHEMA:
        raise ValueError(f"Unsupported coverage schema: {coverage.get('schema')!r}")

    for key in ("roi", "run", "objects", "counters"):
        if key not in coverage:
            raise ValueError(f"Coverage artifact missing required key: {key}")

    if not isinstance(coverage["objects"], Mapping):
        raise ValueError("Coverage objects must be a mapping")
    if not isinstance(coverage["counters"], Mapping):
        raise ValueError("Coverage counters must be a mapping")


def summarize_fifo_occupancy(
    coverage: Mapping[str, Any],
) -> dict[str, dict[str, int]]:
    """Return FIFO occupancy counters keyed by coverage ID."""

    validate_coverage_schema(coverage)
    objects = coverage["objects"]
    counters = coverage["counters"]
    summary: dict[str, dict[str, int]] = {}

    for coverage_id, metadata in objects.items():
        if not isinstance(metadata, Mapping):
            continue
        if metadata.get("kind") != "fifo":
            continue
        raw_counters = counters.get(coverage_id, {})
        if not isinstance(raw_counters, Mapping):
            raw_counters = {}
        entry: MutableMapping[str, int] = {}
        for key in (
            "max_occupancy",
            "configured_rtl_depth",
            "overflow_under_configured_depth",
            "final_occupancy",
        ):
            value = raw_counters.get(key, 0)
            if isinstance(value, int):
                entry[key] = value
        summary[coverage_id] = dict(entry)

    return summary
