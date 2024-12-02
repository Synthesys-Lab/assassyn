import numpy as np
import random

# Parameters
N = 494  # Number of rows in the sparse matrix
L = 10   # Maximum number of non-zero elements per row
MIN = 0  # Minimum value for random generation (int16 range)
MAX = 1023   # Maximum value for random generation (int16 range)

# Function to generate random values in the range [MIN, MAX]
def ran():
    return random.randint(MIN, MAX)

# Function to fill nzval, colind, and x arrays
def fill_val(N, L):
    nzval = []
    colind = []
    x = [ran() for _ in range(N)]  # Generate vector x

    for i in range(N):
        cur_indices = set()  # To ensure unique column indices per row
        row_nzval = []
        row_colind = []

        for _ in range(L):
            # Generate a random column index ensuring uniqueness
            cur_indx = random.randint(0, N - 1)
            while cur_indx in cur_indices:
                cur_indx = random.randint(0, N - 1)
            cur_indices.add(cur_indx)

            # Append non-zero value and column index
            row_nzval.append(ran())
            row_colind.append(cur_indx)

        # Fill nzval and colind arrays
        nzval.extend(row_nzval)
        colind.extend(row_colind)

    return nzval, colind, x

# Write the generated data to a file
def write_to_file(filename, nzval, colind, x):
    with open(filename, 'w') as f:
        f.write("nzval:\n")
        f.write(" ".join(map(str, nzval)) + "\n")
        f.write("colind:\n")
        f.write(" ".join(map(str, colind)) + "\n")
        f.write("x:\n")
        f.write(" ".join(map(str, x)) + "\n")

# Main
if __name__ == "__main__":
    random.seed(8650341)  # Set random seed for reproducibility
    nzval, colind, x = fill_val(N, L)
    write_to_file("./data/ellpack_data.txt", nzval, colind, x)
    print("Data written to ./data/ellpack_data.txt")
