# This is a complete Python code example of multi-key radix sort, used to sort 32-bit binary numbers in groups of 4 bits

def radix_sort_binary(numbers):
    # Sort every 4 bits, 8 rounds in total (32 bits / 4 bits = 8 rounds)
    num_bits = 16
    group_size = 4
    num_groups = num_bits // group_size
    # Start from the lowest 4 bits, sort each group of 4 bits in sequence
    for i in range(num_groups):
        # Count array, used to store the frequency of each 4-bit value
        count = [0] * 16  # 4 binary bits have 16 combinations (0000 to 1111)
        # Calculate the value of current 4-bit group and count
        for num in numbers:
            # Extract the value of current 4-bit group (using bit shift and mask)
            group_value = (num >> (i * group_size)) & 0xF
            count[group_value] += 1

        # Calculate prefix sum to determine the position of each value after sorting
        for j in range(1, 16):
            count[j] += count[j - 1]

        # Place elements into new array based on current 4-bit group sorting result
        sorted_numbers = [0] * len(numbers)
        for num in reversed(numbers):  # Reverse traversal to maintain sorting stability
            group_value = (num >> (i * group_size)) & 0xF
            count[group_value] -= 1
            sorted_numbers[count[group_value]] = num

        # Update the original array with sorted array, prepare for next round of sorting
        numbers = sorted_numbers
        print(f"Round {i} sorting result: ")
        print([f"{num:08x}" for num in sorted_numbers])
        print(f"Round {i} radix result:")
        print([f"{num}" for num in count][1:])
            # print(sorted_numbers)

    return numbers

# Example usage
numbers = [
    0x255c,
    0x41b,
    0x2107,
    0x2380,
    0xc1c,
    0x1440,
    0x28aa,
    0x2dc1,
]

# Execute sorting
sorted_numbers = radix_sort_binary(numbers)

# Output sorting result
sorted_numbers_binary = [f"{num:032b}" for num in sorted_numbers]
print(sorted_numbers_binary)
