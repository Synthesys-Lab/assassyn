"""Self-contained RV32I regressions with explicit architectural expectations."""
import os
from pathlib import Path
import re
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parent / 'src'))
import main as cpu


def immediate(rd, rs1, value, funct3=0, opcode=0x13):
    return ((value & 0xFFF) << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode


def store(rs2, offset, funct3):
    value = offset & 0xFFF
    return ((value >> 5) << 25) | (rs2 << 20) | (funct3 << 12) | ((value & 31) << 7) | 0x23


def jump(rd, offset):
    value = offset & 0x1FFFFF
    return (((value >> 20) & 1) << 31) | (((value >> 1) & 0x3FF) << 21) | \
        (((value >> 11) & 1) << 20) | (((value >> 12) & 0xFF) << 12) | (rd << 7) | 0x6F


def register(rd, rs1, rs2, funct3):
    return (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | 0x33


def run_regressions():
    with tempfile.TemporaryDirectory(prefix='minor-cpu-regression-') as directory:
        root = Path(directory)
        cpu.workspace = str(root) + '/'
        old = Path.cwd()
        os.chdir(root)
        try:
            _, binary, _ = cpu.build_cpu(10, sim_only=True, sim_threshold=500)
            cases = [
                ('subwords', [
                    immediate(2, 0, 0, 0, 3), immediate(3, 0, 1, 4, 3),
                    immediate(4, 0, 0, 1, 3), immediate(5, 0, 2, 5, 3),
                    immediate(6, 0, 0x12), store(6, 1, 0), immediate(7, 0, 0, 2, 3),
                    immediate(6, 0, 0x345), store(6, 2, 1), immediate(8, 0, 0, 2, 3),
                    immediate(9, 0, 1, 0, 3), immediate(10, 0, 2, 5, 3),
                    immediate(11, 0, -1), register(12, 11, 0, 2), register(13, 11, 6, 4),
                ], {2: 0xFFFFFF98, 3: 0xBA, 4: 0xFFFFBA98, 5: 0xFEDC,
                    7: 0xFEDC1298, 8: 0x03451298, 9: 0x12, 10: 0x345,
                    12: 1, 13: 0xFFFFFCBA}),
                ('redirects', [
                    immediate(1, 0, 100), jump(5, 8), immediate(1, 0, 999),
                    immediate(8, 5, 0), immediate(1, 1, 1), jump(6, 4),
                    immediate(7, 7, 1),
                    # Taken branch must squash a store in the prefetched suffix.
                    jump(0, 8), store(1, 0, 2), immediate(9, 0, 0, 2, 3),
                ], {1: 101, 5: 8, 6: 24, 7: 1, 8: 8, 9: 0xFEDCBA98}),
            ]
            for name, program, expected in cases:
                # Drain writeback before architectural halt.
                program = program + [immediate(0, 0, 0)] * 4 + [0x00100073]
                (root / 'workload.exe').write_text(''.join(f'{word:08x}\n' for word in program))
                (root / 'workload.data').write_text('fedcba98\n')
                (root / 'workload.init').write_text('0\n')
                raw = cpu.utils.run_simulator(binary_path=binary)
                if 'ebreak | halt | ecall' not in raw:
                    raise AssertionError(f'{name}: did not halt')
                values = {int(reg): int(value, 16) for reg, value in
                          re.findall(r'writeback\s+\| x(\d+)\s+\| 0x([0-9a-fA-F]+)', raw)}
                for reg, expected_value in expected.items():
                    if values.get(reg) != expected_value:
                        raise AssertionError(f'{name}: x{reg}={values.get(reg)!r}, expected {expected_value:#x}')
                print(f'PASS {name}: {len(expected)} architectural values checked', flush=True)
        finally:
            os.chdir(old)


if __name__ == '__main__':
    run_regressions()
