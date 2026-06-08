#!/usr/bin/env python3

"""Build `.exe/.data/.config` images for minor-cpu from an ELF/dump pair.

The original loader only handled `.text*` and `.data*`, and assumed `.data`
was the first non-text alloc section. CoreMark and similar workloads place
constants in `.rodata/.srodata` and often rely on `.bss/.sbss`, so minor-cpu
needs a single contiguous non-exec image starting at the lowest alloc
non-exec address.
"""

import argparse
import os
import subprocess
import tempfile


parser = argparse.ArgumentParser(
    description="Extract text/data images for the minor-cpu workload format."
)
parser.add_argument("--fname", type=str, required=True, help="Path to a .dump file.")
parser.add_argument("--odir", type=str, default=".", help="Output directory.")
parser.add_argument("--exit-tohost", action="store_true", help="Unused legacy flag.")
args = vars(parser.parse_args())


def derive_elf_path(fname: str) -> str:
    candidates = [fname]
    if fname.endswith(".dump"):
        base = fname[:-5]
        candidates = [base, base + ".elf"]
        if base.endswith(".riscv"):
            candidates.extend([base[:-6], base[:-6] + ".elf"])
    for path in candidates:
        if os.path.exists(path):
            return path
    return candidates[0]


def derive_output_stem(fname: str) -> str:
    bin_name = os.path.split(fname)[-1]
    if bin_name.endswith(".riscv.dump"):
        return bin_name[: -len(".riscv.dump")]
    if bin_name.endswith(".dump"):
        return bin_name[: -len(".dump")]
    return bin_name


def parse_sections(elf_path: str) -> list[dict]:
    raw = subprocess.check_output(
        ["riscv64-unknown-elf-readelf", "-S", "-W", elf_path]
    ).decode("utf-8")

    res = []
    for line in raw.splitlines():
        toks = line.strip().split()
        if not toks:
            continue
        if toks[0] == "[":
            toks = [toks[0] + toks[1]] + toks[2:]
        if not toks[0].startswith("[") or not toks[0].endswith("]"):
            continue
        sec_id = toks[0][1:-1]
        if not sec_id.isdigit():
            continue
        if len(toks) < 8:
            continue

        res.append(
            {
                "name": toks[1],
                "type": toks[2],
                "addr": int(toks[3], 16),
                "off": int(toks[4], 16),
                "size": int(toks[5], 16),
                "flags": toks[7],
            }
        )
    return res


def dump_section_bytes(elf_path: str, section_name: str) -> bytes:
    with tempfile.TemporaryDirectory() as tmpdir:
        out_path = os.path.join(tmpdir, "section.bin")
        subprocess.check_call(
            [
                "riscv64-unknown-elf-objcopy",
                f"--dump-section",
                f"{section_name}={out_path}",
                elf_path,
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        with open(out_path, "rb") as f:
            return f.read()


def build_image(elf_path: str, sections: list[dict]) -> tuple[int, bytearray]:
    if not sections:
        return 0, bytearray()

    base = min(sec["addr"] for sec in sections)
    end = max(sec["addr"] + sec["size"] for sec in sections)
    image = bytearray(end - base)

    for sec in sections:
        if sec["size"] == 0:
            continue
        if sec["type"] == "NOBITS":
            continue
        payload = dump_section_bytes(elf_path, sec["name"])
        assert len(payload) == sec["size"], (
            f"Section size mismatch for {sec['name']}: "
            f"read {len(payload)} bytes, expected {sec['size']}"
        )
        start = sec["addr"] - base
        image[start : start + sec["size"]] = payload

    return base, image


def write_word_image(path: str, image: bytearray, max_words: int = 1 << 16) -> None:
    if not image:
        open(path, "w").write("")
        return

    if len(image) % 4 != 0:
        image.extend(b"\x00" * (4 - (len(image) % 4)))

    words = len(image) // 4
    assert words <= max_words, f"Image too large for SRAM: {words} words > {max_words}"

    with open(path, "w") as f:
        for i in range(0, len(image), 4):
            word = int.from_bytes(image[i : i + 4], byteorder="little", signed=False)
            f.write(f"{word:08x}\n")


fname = args["fname"]
elf_path = derive_elf_path(fname)
stem = derive_output_stem(fname)

print(f"Extracting {fname} (ELF: {elf_path})...")

sections = parse_sections(elf_path)

text_sections = [
    sec for sec in sections if "A" in sec["flags"] and "X" in sec["flags"] and sec["size"] > 0
]
data_sections = [
    sec for sec in sections if "A" in sec["flags"] and "X" not in sec["flags"] and sec["size"] >= 0
]

assert text_sections, f"No alloc+exec sections found in {elf_path}"

text_base, text_image = build_image(elf_path, text_sections)

if data_sections:
    data_base, data_image = build_image(elf_path, data_sections)
else:
    data_base, data_image = 0, bytearray()

data_offset = data_base - text_base

ofile = os.path.join(args["odir"], stem)
with open(ofile + "config", "w") as f:
    f.write(f'{{ "offset": {hex(text_base)}, "data_offset": {hex(data_offset)} }}')

write_word_image(ofile + "exe", text_image)
write_word_image(ofile + "data", data_image)
