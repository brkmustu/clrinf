# Mapper Benchmark Performance Report (AutoMapper vs Static Extension Mapper)

This benchmark compares the execution speed (latency) and memory allocations of **AutoMapper v13.0.1** against our zero-dependency **Static Extension Mapper** in .NET 10.

## Benchmark Environment
- **OS:** Windows 11 (10.0.26200.8655)
- **CPU:** 13th Gen Intel Core i7-13700 2.10GHz (16 physical cores, 24 logical cores)
- **Runtime:** .NET 10.0.10 (Release Mode)
- **Benchmark Tool:** BenchmarkDotNet v0.15.8

---

## Results Summary

| Method | Mean | Error | StdDev | Ratio | Gen0 | Gen1 | Allocated | Alloc Ratio | Rank |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |
| **SingleObject_ExtensionMapper** | **3.828 ns** | **0.1026 ns** | **0.0960 ns** | **0.14** | **0.0036** | **-** | **56 B** | **1.00** | **1 🥇** |
| **SingleObject_AutoMapper** | **27.355 ns** | **0.5639 ns** | **0.5791 ns** | **1.00** | **0.0035** | **-** | **56 B** | **1.00** | **2** |
| **List1000_ExtensionMapper** | **5,937.070 ns** | **10.1321 ns** | **8.9819 ns** | **217.13** | **4.0894** | **0.8163** | **64,128 B** | **1,145.14** | **3 🥇** |
| **List1000_AutoMapper** | **6,481.635 ns** | **97.0785 ns** | **75.7925 ns** | **237.04** | **4.6234** | **0.9232** | **72,600 B** | **1,296.43** | **4** |

---

## Key Highlights

1. **7.15x Faster Execution (Latency):**
   - Single object mapping with AutoMapper takes **27.35 ns**, while our Static Extension Mapper takes **3.82 ns** (7.15x faster).
2. **12% Memory Allocation Savings:**
   - For a 1,000-item collection transformation, AutoMapper allocates **72,600 bytes (72.6 KB)** whereas the Extension Mapper allocates only **64,128 bytes (64.1 KB)**.
3. **Zero Cold Start & No Runtime Reflection:**
   - AutoMapper compiles dynamic expression trees and maintains a runtime delegate cache. The Extension Mapper is compiled into standard IL and benefits directly from JIT inlining.
