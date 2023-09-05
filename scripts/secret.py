import random

p = 363967321904221003
q = 7696033
g = 165950041202038920
h = 96429580695728554


x = random.getrandbits(64)
print("x =", x)
y1 = pow(g, x, p)
y2 = pow(h, x, p)
print("y1 =", y1)
print("y2 =", y2)


k = random.getrandbits(64)
print("k =", k)
r1 = pow(g, k, p)
r2 = pow(h, k, p)
print("r1 =", r1)
print("r2 =", r2)


# c = random.getrandbits(64) % q
# print("c =", c)

# s = (k - c * x) % q
# print("s =", s)


# print(r1, pow(g, s, p) * pow(y1, c, p) % p)
# print(r2, pow(h, s, p) * pow(y2, c, p) % p)
