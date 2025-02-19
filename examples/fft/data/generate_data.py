import random

# Number of 64-bit numbers to generate
num_values = 2048

# Generate and write each 64-bit number in hexadecimal to a file, ensuring 64-bit padding
with open("fft_data.data", "w") as file:
    for _ in range(num_values):
        file.write(f"{random.randint(0, 2**64 - 1):016x}\n")