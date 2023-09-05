import random

from sympy import isprime, primefactors


while True:
    p = random.getrandbits(64)
    if isprime(p):
        break
print(p)
print(primefactors(p - 1))
