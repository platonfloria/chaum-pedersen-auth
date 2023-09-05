import random


p = 363967321904221003
q = 7696033
print((p - 1) // q)
for key in "gh":
    val = 1
    while val % p == 1:
        val = pow(random.getrandbits(64), (p - 1) // q, p)
        print(f"{key} = {val}")
