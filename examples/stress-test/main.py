print("Starting runtime stress test...")

# 1. Fast iterative Fibonacci
def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

fib_result = fib(100_000)  # Fast but big

# 2. Prime sieve up to N (efficient)
def sieve(limit):
    is_prime = [True] * (limit + 1)
    is_prime[0:2] = [False, False]
    for i in range(2, int(limit ** 0.5) + 1):
        if is_prime[i]:
            for j in range(i * i, limit + 1, i):
                is_prime[j] = False
    return [i for i, val in enumerate(is_prime) if val]

primes = sieve(100_000)
prime_count = len(primes)

# 3. 2D matrix sum (simulate numeric ops)
matrix = [[(i * j) % 97 for j in range(200)] for i in range(200)]
matrix_sum = sum(sum(row) for row in matrix)

# Final output
print("Finished.")
print("Fibonacci(100000) last 6 digits:", fib_result % 1_000_000)
print("Total primes under 100000:", prime_count)
print("Sum of matrix elements:", matrix_sum)


"""
Expected output:

    Starting runtime stress test...
    Finished.
    Fibonacci(100000) last 6 digits: 746875
    Total primes under 100000: 9592
    Sum of matrix elements: 1881249
"""
