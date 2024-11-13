def read_and_reformat_file(input_filename, output_filename):
    with open(input_filename, 'r') as infile:
        lines = infile.readlines()
        
        # Extract nzval, colind, and x from the file
        nzval_index = lines.index("nzval:\n") + 1
        colind_index = lines.index("colind:\n") + 1
        x_index = lines.index("x:\n") + 1
        
        nzval = list(map(int, lines[nzval_index].split()))
        colind = list(map(int, lines[colind_index].split()))
        x = list(map(int, lines[x_index].split()))
        
    with open(output_filename, 'w') as outfile:
        # Write nzval data with //nzval as a comment
        outfile.write("//nzval\n")
        for val in nzval:
            outfile.write(f"{format(val & 0xFFFFFFFF, '08x')}\n")
        
        # Write colind data with //colind as a comment
        outfile.write("//colind\n")
        for val in colind:
            outfile.write(f"{format(val & 0xFFFFFFFF, '08x')}\n")
        
        # Write x data with //x as a comment
        outfile.write("//x\n")
        for val in x:
            outfile.write(f"{format(val & 0xFFFFFFFF, '08x')}\n")

if __name__ == "__main__":
    input_filename = "./data/ellpack_data.txt"
    output_filename = "./data/ellpack_data_reformatted.data"
    read_and_reformat_file(input_filename, output_filename)
    print(f"Data reformatted and written to {output_filename}")
