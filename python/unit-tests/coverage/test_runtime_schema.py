"""Executable expectations for semantic coverage JSON helpers."""

import json

from assassyn.verification.coverage import (  # type: ignore
    load_coverage,
    summarize_fifo_occupancy,
    validate_coverage_schema,
)


def test_validate_minimal_coverage_schema(tmp_path):
    """Coverage readers accept the versioned semantic coverage schema."""

    coverage_path = tmp_path / "coverage.json"
    coverage_path.write_text(
        json.dumps(
            {
                "schema": "assassyn.semantic_coverage.v1",
                "roi": {"start_cycle": 3, "end_cycle": 12},
                "run": {"sim_threshold": 20, "covered_cycles": 10},
                "objects": {
                    "fifo:consumer.data": {
                        "kind": "fifo",
                        "module": "consumer",
                        "port": "data",
                        "loc": "example.py:17",
                        "expr": "data.pop()",
                    }
                },
                "counters": {
                    "fifo:consumer.data": {
                        "push": 4,
                        "pop": 3,
                        "max_occupancy": 2,
                        "configured_rtl_depth": 1,
                        "overflow_under_configured_depth": 1,
                    }
                },
            }
        ),
        encoding="utf-8",
    )

    coverage = load_coverage(coverage_path)
    validate_coverage_schema(coverage)

    summary = summarize_fifo_occupancy(coverage)
    assert summary["fifo:consumer.data"]["max_occupancy"] == 2
    assert summary["fifo:consumer.data"]["configured_rtl_depth"] == 1
    assert summary["fifo:consumer.data"]["overflow_under_configured_depth"] == 1
