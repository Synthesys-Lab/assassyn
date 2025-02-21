import random

# Number of 64-bit numbers to generate
num_values = 2048

# Generate and write each 64-bit number in hexadecimal to a file, ensuring 64-bit padding
with open("fft_data.data", "w") as file:
    for _ in range(num_values):
        upper_32 = random.randint(0, (2**3) - 1)
        lower_32 = random.randint(0, (2**3) - 1)
        full_64 = (upper_32 << 32) | lower_32
        file.write(f"{full_64:016x}\n")

