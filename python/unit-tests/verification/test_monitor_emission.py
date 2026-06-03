"""Tests for generated translation-validation monitors."""

from copy import deepcopy

from assassyn.verification.checks import check_model_consistency  # type: ignore
from assassyn.verification.emit import render_monitor  # type: ignore
from assassyn.verification.model import (  # type: ignore
    ArrayReadPortTransition,
    ArrayTransition,
    ArrayWritePortTransition,
    AsyncCallTransition,
    FIFOTransition,
    ModuleTransition,
    RTLSignalMap,
    TriggerTransition,
    ValidationModel,
)


def build_model() -> ValidationModel:
    """Build a small validation model with one trigger and one FIFO."""

    model = ValidationModel()
    model.modules["module:Target"] = ModuleTransition(
        coverage_id="module:Target",
        module="Target",
        fire_signal="inst_target.executed",
        event_count_signal="target_trigger_counter_inst.count",
    )
    model.triggers["module:Target"] = TriggerTransition(
        coverage_id="module:Target",
        module="Target",
        rtl_count_signal="target_trigger_counter_inst.count",
        rtl_delta_signal="target_trigger_counter_delta",
        width=3,
    )
    model.fifos["fifo:Target.data"] = FIFOTransition(
        coverage_id="fifo:Target.data",
        module="Target",
        port="data",
        configured_depth_log2=2,
        configured_depth=4,
        rtl=RTLSignalMap(
            count_signal="fifo_target_data_inst.multi_element_fifo.count",
            count_width=3,
            push_valid_signal="fifo_target_data_push_valid",
            valid_signal="fifo_target_data_pop_valid",
            ready_signal="fifo_target_data_push_ready",
            pop_ready_signal="fifo_target_data_pop_ready",
            data_signal="fifo_target_data_pop_data",
            data_width=32,
        ),
    )
    model.async_calls["async:Driver->Target:0"] = AsyncCallTransition(
        coverage_id="async:Driver->Target:0",
        caller="Driver",
        callee="Target",
        fifo_ids=("fifo:Target.data",),
    )
    model.arrays["array:state"] = ArrayTransition(
        coverage_id="array:state",
        array="state",
        depth=4,
        index_width=2,
        data_width=8,
        write_ports=(
            ArrayWritePortTransition(
                writer="Driver",
                port_index=0,
                write_enable_signal="_Driver_state_w_port0",
                write_index_signal="_Driver_state_widx_port0",
                write_data_signal="_Driver_state_wdata_port0",
                next_value_signal="state.mem[_Driver_state_widx_port0]",
            ),
        ),
        read_ports=(
            ArrayReadPortTransition(
                reader="Driver",
                port_index=0,
                read_index_signal="_Driver_state_ridx_port0",
                read_data_signal="_state_rdata_port0",
            ),
        ),
    )
    return model


def test_monitor_emits_bind_and_fifo_assertions():
    """Rendered monitor includes concrete RTL paths and safety assertions."""

    monitor = render_monitor(build_model())

    assert "bind Top translation_validation_monitor" in monitor
    assert "bind fifo translation_validation_fifo_monitor" in monitor
    assert "bind trigger_counter translation_validation_trigger_monitor" in monitor
    assert "fifo push_valid unknown" in monitor
    assert "fifo pop_data unknown" in monitor
    assert "trigger_counter trigger count overflow" in monitor
    assert "_Driver_state_w_port0" in monitor
    assert "state.mem[_Driver_state_widx_port0]" in monitor
    assert "next-cycle payload mismatch" in monitor
    assert "same-cycle write visible through read port" in monitor
    assert "longint unsigned tv_assert_0_activations" in monitor
    assert "translation_validation_assertion name=fifo.push_valid_known" in monitor
    assert "translation_validation_assertion name=array:state.w0.next_cycle_payload" in monitor
    assert "translation_validation_assertion name=trigger_counter.count_bounded" in monitor


def test_model_consistency_catches_mutated_missing_fifo():
    """Static checks catch model mutations before monitor emission."""

    model = build_model()
    assert not check_model_consistency(model)

    mutated = deepcopy(model)
    del mutated.fifos["fifo:Target.data"]

    errors = check_model_consistency(mutated)
    assert errors
    assert "references missing FIFO fifo:Target.data" in errors[0]
