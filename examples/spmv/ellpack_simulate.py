import numpy as np

# Parameters
N = 494  # Number of rows in the sparse matrix
L = 10   # Maximum number of non-zero elements per row

def read_from_file(filename):
    with open(filename, 'r') as f:
        lines = f.readlines()
        
        # Extract nzval, colind, and x from the file
        nzval_index = lines.index("nzval:\n") + 1
        colind_index = lines.index("colind:\n") + 1
        x_index = lines.index("x:\n") + 1
        
        nzval = list(map(int, lines[nzval_index].split()))
        colind = list(map(int, lines[colind_index].split()))
        x = list(map(int, lines[x_index].split()))
        
    return nzval, colind, x

def ellpack(nzval, colind, x, N, L):
    # Initialize output vector
    out = [0] * N

    # Perform the ELLPACK matrix-vector multiplication
    for i in range(N):
        sum_val = 0
        print(f"Row {i} computation:")
        for j in range(L):
            idx = j + i * L
            product = nzval[idx] * x[colind[idx]]
            sum_val += product
            print(f"  nzval[{idx}] * x[{colind[idx]}] = {nzval[idx]} * {x[colind[idx]]} = {product}")
        out[i] = sum_val
        print(f"  Sum for row {i}: {sum_val}\n")
    
    return out

def main():
    # Read data from file
    filename = "./data/ellpack_data.txt"
    nzval, colind, x = read_from_file(filename)

    # Perform the ELLPACK matrix-vector multiplication
    result = ellpack(nzval, colind, x, N, L)

    # Print the output vector
    print("Output vector:")
    print(result)

if __name__ == "__main__":
    main()
