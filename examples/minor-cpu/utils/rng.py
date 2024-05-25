import random

res = 0
for i in range(100):
    v = random.randint(1, 12345)
    res += v
    print('%x' % v)

print('// %x' % res)
