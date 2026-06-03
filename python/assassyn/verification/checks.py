"""Static checks for normalized validation models."""

from __future__ import annotations

from .model import ValidationModel


def check_model_consistency(model: ValidationModel) -> list[str]:
    """Return a list of consistency errors for *model*."""

    errors: list[str] = []
    for async_id, async_call in model.async_calls.items():
        if f"module:{async_call.callee}" not in model.modules:
            errors.append(f"{async_id} targets missing module {async_call.callee}")
        for fifo_id in async_call.fifo_ids:
            if fifo_id not in model.fifos:
                errors.append(f"{async_id} references missing FIFO {fifo_id}")
    return errors
