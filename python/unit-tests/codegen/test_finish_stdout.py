"""FINISH must explicitly flush stdout before exiting the simulator process."""
import shutil
import subprocess

import pytest

from assassyn.codegen.simulator._expr.intrinsics import _codegen_finish


def test_finish_flushes_before_exit():
    code = _codegen_finish(None, None)
    assert code.index('Write::flush') < code.index('std::process::exit(0)')


def test_finish_preserves_output_without_newline(tmp_path):
    if shutil.which('rustc') is None:
        pytest.skip('rustc is required to execute generated simulator code')
    code = _codegen_finish(None, None)
    source = tmp_path / 'finish.rs'
    source.write_text('fn main() { print!("final-marker"); ' + code + ' }')
    binary = tmp_path / 'finish'
    subprocess.run(['rustc', str(source), '-o', str(binary)], check=True, capture_output=True)
    result = subprocess.run([str(binary)], check=True, capture_output=True, text=True)
    assert result.stdout == 'final-marker'
    assert result.stderr == ''
