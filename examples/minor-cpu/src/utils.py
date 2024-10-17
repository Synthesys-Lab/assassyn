
from assassyn import utils

def check(raw):
    data_path = f'{utils.repo_path()}/examples/minor-cpu/workloads/0to100.data'
    with open(data_path, 'r') as f:
        data = []
        for line in f:
            line = line.split('//')[0].strip()
            if line and not line.startswith('@'):
                try:
                    data.append(int(line, 16))
                except ValueError:
                    print(f"Warning: Skipping invalid line: {line}")


    accumulator = 0
    ideal_accumulator = 0
    data_index = 0

    for line in raw.split('\n'):

        if 'writeback' in line and 'x14 = ' in line:
            loaded = int(line.split('=')[-1].strip(), 16)
            assert data[data_index] == loaded, f"Data mismatch at step {data_index + 1}: {hex(data[data_index])} != {hex(loaded)}"

        if 'writeback' in line and 'x15 = ' in line:
            addr = int(line.split('=')[-1].strip(), 16)
            if addr == 0 or addr == 0xb8:
                continue
            assert 0xb8 + (data_index + 1) * 4 == addr, f"Address mismatch at step {data_index + 1}: {hex(addr)} != {hex(0xb8 + (data_index + 1) * 4)}"

        if 'writeback' in line and 'x10 = ' in line:
            value = int(line.split('=')[-1].strip(), 16)
            if value != accumulator:
                accumulator = value
                if data_index < len(data):
                    ideal_accumulator += data[data_index]
                    assert accumulator == ideal_accumulator, f"Mismatch at step {data_index + 1}: CPU {accumulator} != Reference {ideal_accumulator}"
                    data_index += 1

    assert data_index == 100, f"Data index mismatch: {data_index} != 100"

    print(f"Final CPU sum: {accumulator} (0x{accumulator:x})")
    print(f"Final ideal sum: {ideal_accumulator} (0x{ideal_accumulator:x})")
    print(f"Final difference: {accumulator - ideal_accumulator}")
