"""Main package entrypoint for multifile-test."""
from utils import add
from subpkg import greet
print("Main package loaded")

# Test importing from utils
result = add(5, 3)
print(f"add(5, 3) = {result}")

# Test importing from subpackage
greet("Venice")

"""
Expected output:

    Utils module loaded
    Main package loaded
    add(5, 3) = 8
    test says Hello, Venice
"""
