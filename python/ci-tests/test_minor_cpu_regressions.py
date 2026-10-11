"""Exercise the example's branch epochs, forwarding and subword memory path."""
from pathlib import Path
import subprocess
import sys


def test_minor_cpu_regressions():
    source = Path(__file__).resolve().parents[2] / 'examples/minor-cpu/test_regressions.py'
    result = subprocess.run([sys.executable, str(source)], check=True,
                            capture_output=True, text=True, timeout=180)
    assert 'PASS subwords: 10 architectural values checked' in result.stdout
    assert 'PASS redirects: 6 architectural values checked' in result.stdout
