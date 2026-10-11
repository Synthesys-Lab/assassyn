"""Test host-side minor-CPU helpers without elaborating the hardware DSL."""
import argparse
import ast
from contextlib import nullcontext
import csv
import os
from pathlib import Path
import re
import shutil
import subprocess
from types import SimpleNamespace
from unittest.mock import Mock

import pytest

SOURCE = Path(__file__).resolve().parents[2] / 'examples/minor-cpu/src/main.py'


def load_helpers(source=SOURCE):
    """Load actual host helpers in isolation from the example's generic imports."""
    tree = ast.parse(source.read_text())
    names = {'check', 'parse_args', 'extract_last_cycle', 'extract_kernel_cycles',
             'measure_cycles', 'cp_required', 'init_workspace', 'resolve_workload_case',
             'write_cycle_summary', 'positive_int', 'run_cpu', 'run_workloads'}
    nodes = [node for node in tree.body if isinstance(node, ast.FunctionDef)
             and node.name in names]
    scope = dict(argparse=argparse, csv=csv, os=os, re=re, shutil=shutil,
                 subprocess=subprocess, WORKLOAD_ALIASES={'coremask': 'coremark'})
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(source), 'exec'), scope)
    return scope


@pytest.fixture
def helpers(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    scope = load_helpers()
    scope.update(workspace=str(tmp_path), current_path=str(tmp_path))
    return scope


@pytest.mark.parametrize('raw', ['FAIL\nebreak | halt | ecall\n', 'trap\n', ''])
def test_checker_failure_is_not_overridden_by_halt(helpers, tmp_path, raw):
    (tmp_path / 'raw.log').write_text(raw)
    def checker(*args, check=False):
        if check:
            raise subprocess.CalledProcessError(1, ['checker'])
        return SimpleNamespace(returncode=1)
    run = Mock(side_effect=checker)
    helpers['subprocess'] = SimpleNamespace(run=run)
    with pytest.raises(subprocess.CalledProcessError):
        helpers['check']()
    assert run.call_args.kwargs['check'] is True


def test_custom_checker_is_authoritative(helpers, tmp_path):
    (tmp_path / 'workload.sh').touch()
    run = Mock(return_value=SimpleNamespace(returncode=0))
    helpers['subprocess'] = SimpleNamespace(run=run)
    helpers['check']()
    assert run.call_args.args[0] == [str(tmp_path / 'workload.sh'), 'raw.log',
                                    str(tmp_path / 'workload.data')]
    assert run.call_args.kwargs['check'] is True


def test_default_checker_success(helpers):
    run = Mock(return_value=SimpleNamespace(returncode=0))
    helpers['subprocess'] = SimpleNamespace(run=run)
    helpers['check']()
    assert run.call_args.kwargs['check'] is True


def test_sim_only_and_cycle_budget(helpers):
    args = helpers['parse_args'](['--sim-only', '--sim-threshold', '1200', '0to100'])
    assert args.sim_only and args.sim_threshold == 1200
    assert args.workloads == ['0to100']
    assert helpers['parse_args']([]).sim_threshold == 600000


@pytest.mark.parametrize('value', ['0', '-1', 'not-a-number'])
def test_invalid_cycle_budget(helpers, value):
    with pytest.raises(SystemExit):
        helpers['parse_args'](['--sim-threshold', value])


def test_missing_asset_fails(helpers, tmp_path):
    with pytest.raises(FileNotFoundError):
        helpers['cp_required'](str(tmp_path / 'missing.exe'), str(tmp_path / 'copy'))


def test_packaged_alias(helpers, tmp_path):
    (tmp_path / 'coremark.exe').touch()
    (tmp_path / 'coremark.config').touch()
    assert helpers['resolve_workload_case'](str(tmp_path), 'coremask') == 'coremark'


@pytest.mark.parametrize('kind', ['tree', 'baremetal', 'disc'])
def test_source_paths_are_rejected_without_launch(helpers, tmp_path, kind):
    if kind == 'tree':
        source = tmp_path / 'coremark'
        source.mkdir()
        for name in ['core_list_join.c', 'core_main.c', 'core_matrix.c',
                     'core_state.c', 'core_util.c']:
            (source / name).touch()
    else:
        source = tmp_path / ('coremark_baremetal.c' if kind == 'baremetal' else 'test.ds')
        source.touch()
    helpers['subprocess'] = SimpleNamespace(run=Mock())
    with pytest.raises(ValueError, match='Unsupported workload path'):
        helpers['resolve_workload_case'](str(tmp_path), str(source))
    helpers['subprocess'].run.assert_not_called()


@pytest.mark.parametrize('rtl_enabled,rtl_path', [(False, 'rtl'), (True, 'rtl'), (True, None)])
def test_runner_checks_each_enabled_backend(helpers, tmp_path, rtl_enabled, rtl_path):
    (tmp_path / 'workload.config').write_text('{"offset":0, "data_offset":0}\n')
    utils = SimpleNamespace(run_simulator=Mock(return_value='Cycle @10.00: sim\n'),
                            run_verilator=Mock(return_value='Cycle @12.00: rtl\n'))
    helpers['utils'] = utils
    logs = []
    helpers['check'] = lambda: logs.append((tmp_path / 'raw.log').read_text())
    result = helpers['run_cpu'](nullcontext(), 'simulator', rtl_path,
                                run_verilator_check=rtl_enabled)
    assert result == (10, 'full')
    utils.run_simulator.assert_called_once_with(binary_path='simulator')
    expected_logs = ['Cycle @10.00: sim\n']
    if rtl_enabled and rtl_path:
        utils.run_verilator.assert_called_once_with(rtl_path)
        expected_logs.append('Cycle @12.00: rtl\n')
    else:
        utils.run_verilator.assert_not_called()
    assert logs == expected_logs
    assert not (tmp_path / 'raw.log').exists()


@pytest.mark.parametrize('failed_backend', ['sim', 'rtl'])
def test_runner_preserves_failed_backend_log(helpers, tmp_path, failed_backend):
    (tmp_path / 'workload.config').write_text('{"offset":0, "data_offset":0}\n')
    utils = SimpleNamespace(run_simulator=Mock(return_value='sim log'),
                            run_verilator=Mock(return_value='rtl log'))
    helpers['utils'] = utils
    failure = subprocess.CalledProcessError(1, ['checker'])
    helpers['check'] = Mock(side_effect=[failure] if failed_backend == 'sim' else [None, failure])
    with pytest.raises(subprocess.CalledProcessError):
        helpers['run_cpu'](nullcontext(), 'simulator', 'rtl')
    assert (tmp_path / 'raw.log').read_text() == f'{failed_backend} log'
    if failed_backend == 'sim':
        utils.run_verilator.assert_not_called()


def test_workload_loop_forwards_backend_choice(helpers):
    helpers['init_workspace'] = Mock()
    helpers['run_cpu'] = Mock(return_value=(10, 'full'))
    system = object()
    assert helpers['run_workloads'](system, 'simulator', 'rtl', 'assets', ['0to100'], False) == [
        ('0to100', 10, 'full')]
    helpers['init_workspace'].assert_called_once_with('assets', '0to100')
    helpers['run_cpu'].assert_called_once_with(system, 'simulator', 'rtl', run_verilator_check=False)


def test_cycle_measurements(helpers):
    assert helpers['measure_cycles']('Cycle @10.00: ordinary\n') == (10, 'full')
    raw = ('Cycle @3.00: [D] imm: 0xb00\n'
           'Cycle @10.00: [E] decoded         | raw: 0xb0002573\n'
           'Cycle @30.00: [E] decoded         | raw: 0xb00025f3\n')
    assert helpers['measure_cycles'](raw) == (20, 'csr_b00')


def test_csv_summary(helpers, tmp_path):
    target = helpers['write_cycle_summary']([('0to100', 12, 'full')])
    with open(target, newline='') as handle:
        rows = list(csv.reader(handle))
    assert rows == [['workload', 'minor_cpu_cycles', 'cycle_source'], ['0to100', '12', 'full']]


@pytest.mark.parametrize('body,expected', [
    ('Cycle @1: [D] raw: 0x02301063\nCycle @2: [E] ebreak | halt | ecall\n', 1),
    ('Cycle @1: [E] decoded | raw: 0x02301063\n', 1),
    ('Cycle @1: [E] decoded | raw: 0x02301063\nCycle @2: [E] ebreak | halt | ecall\n', 0),
])
def test_pass_marker_requires_execution_and_halt(tmp_path, body, expected):
    raw = tmp_path / 'raw.log'
    raw.write_text(body)
    script = SOURCE.parent.parent / 'utils/find_pass.sh'
    result = subprocess.run(['sh', str(script), str(raw)], check=False)
    assert result.returncode == expected
