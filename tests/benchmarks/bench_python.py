#!/usr/bin/env python3
"""
Heavy Benchmark Suite for Python
Goal: Compare with SFX performance
"""

import time
from decimal import Decimal, getcontext

# Set decimal precision to match SFX's arbitrary precision
getcontext().prec = 50

class Item:
    def __init__(self):
        self.value = 0

class Calculator:
    def __init__(self):
        self.result = 0
    
    def add(self, val):
        self.result = self.result + val

def main():
    print("=== PYTHON HEAVY BENCHMARK ===")
    print()

    # -------------------------------------------------
    # TEST 1: The Loop (1,000,000 iterations)
    # Stress: Integer/Decimal Math
    # -------------------------------------------------
    print("1. Loop (1,000,000 ops)...")
    
    start_time = time.perf_counter()
    sum_val = Decimal(0)
    for _ in range(1000000):
        sum_val = sum_val + Decimal(1)
    end_time = time.perf_counter()
    
    elapsed_loop = end_time - start_time
    print(f"   Done. Sum: {sum_val}")
    print(f"   Time: {elapsed_loop:.10f} seconds")
    print()

    # -------------------------------------------------
    # TEST 2: Allocation (10,000 Objects)
    # Stress: Memory Allocator, Object creation
    # -------------------------------------------------
    print("2. Allocation (10,000 objects)...")
    
    start_time = time.perf_counter()
    for _ in range(10000):
        thing = Item()
        thing.value = 100
    end_time = time.perf_counter()
    
    elapsed_alloc = end_time - start_time
    print("   Done.")
    print(f"   Time: {elapsed_alloc:.10f} seconds")
    print()

    # -------------------------------------------------
    # TEST 3: The Dispatcher (100,000 Calls)
    # Stress: Method Lookup, Method Binding
    # -------------------------------------------------
    print("3. Dispatch (100,000 calls)...")
    
    calc = Calculator()
    
    start_time = time.perf_counter()
    for _ in range(100000):
        calc.add(1)
    end_time = time.perf_counter()
    
    elapsed_dispatch = end_time - start_time
    print(f"   Done. Result: {calc.result}")
    print(f"   Time: {elapsed_dispatch:.10f} seconds")
    print()

    # -------------------------------------------------
    # SUMMARY
    # -------------------------------------------------
    total_time = elapsed_loop + elapsed_alloc + elapsed_dispatch
    print("=== SUMMARY ===")
    print(f"Loop:       {elapsed_loop:.10f}s")
    print(f"Allocation: {elapsed_alloc:.10f}s")
    print(f"Dispatch:   {elapsed_dispatch:.10f}s")
    print(f"Total:      {total_time:.10f}s")

if __name__ == "__main__":
    main()
