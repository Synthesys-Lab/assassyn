# 这是一个多关键字基数排序的完整 Python 代码示例，用于对 32 位二进制数按每 4 位一组进行排序

def radix_sort_binary(numbers):
    # 每 4 位进行排序，一共需要 8 轮（32位 / 4位 = 8轮）
    num_bits = 32
    group_size = 4
    num_groups = num_bits // group_size
    # 从最低位的4位开始，每组4位依次排序
    for i in range(num_groups):
        # 计数数组，用于存储每组4位的值出现的次数
        count = [0] * 16  # 4位二进制有16种组合（0000到1111）
        # 计算当前4位组的值并计数
        for num in numbers:
            # 提取当前 4 位组的值（使用位移和掩码）
            group_value = (num >> (i * group_size)) & 0xF
            count[group_value] += 1

        # 计算前缀和，用于确定每个值在排序后的位置
        for j in range(1, 16):
            count[j] += count[j - 1]

        # 根据当前4位组的排序结果，将元素放入新的数组
        sorted_numbers = [0] * len(numbers)
        for num in reversed(numbers):  # 倒序遍历，保证排序稳定性
            group_value = (num >> (i * group_size)) & 0xF
            count[group_value] -= 1
            sorted_numbers[count[group_value]] = num

        # 将排序后的数组更新到原始数组中，准备进行下一轮排序
        numbers = sorted_numbers
        if i == 0:
            print(f"第 {i} 轮排序结果: ")
            # print([f"{num:032b}" for num in sorted_numbers])
            print(sorted_numbers)

    return numbers

# 示例用法
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

# 执行排序
sorted_numbers = radix_sort_binary(numbers)

# 输出排序结果
sorted_numbers_binary = [f"{num:032b}" for num in sorted_numbers]
# print(sorted_numbers_binary)
