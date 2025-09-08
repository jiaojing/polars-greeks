# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

High-performance Black-Scholes options pricing library with Python bindings via Polars. Built as a Rust workspace with two main crates:

- `rust-core` - Pure Rust Black-Scholes implementation with comprehensive Greeks calculations
- `rust-py-bindings` - Polars plugin providing vectorized Python interface

## Core Architecture

**Data Flow**: Python DataFrame → Polars Series → Rust calculation engine → Polars struct output

**Key Components**:
- `BlackScholesModel` - Core pricing engine with lazy-initialized cached calculations
- `GreeksVec` - Vectorized Greeks accumulator for batch processing  
- `polars_greeks` plugin - Zero-copy integration with Polars ecosystem

## Build Commands

```bash
# Development build
cargo build

# Optimized release build  
cargo build --release

# Run Rust tests
cargo test

# Python development install (from rust-py-bindings/)
maturin develop

# Python tests
python -m pytest tests/

# Performance benchmarks
python -m pytest tests/test_performance.py -v
```

## Testing Strategy

- Rust unit tests in each crate verify mathematical correctness
- Python integration tests validate plugin behavior and edge cases
- Performance tests ensure vectorization efficiency
- Mathematical accuracy tests verify put-call parity and known option values

## Key Design Patterns

**Lazy Calculation Caching**: `OnceCell` fields in `BlackScholesModel` cache expensive operations (d1, d2, N(d1), φ(d1)) computed on first access.

**Zero-Copy Integration**: Direct Series→Series transformations avoid Python object conversion overhead.

**Selective Computation**: `GreeksFlags` enables calculating only requested Greeks, avoiding unnecessary work.

**Memory Efficiency**: Pre-allocated vectors with exact capacity prevent reallocations during batch processing.

## Development Notes

- Python package name is `polars-greeks` but module name is `polars_greeks`
- All Greeks calculations support both Call/Put option types
- Finite difference methods available for vanna/volga validation
- Uses `statrs` crate for high-precision statistical functions
- Polars integration requires struct output schema inference

## File Organization

```
crates/rust-core/src/
  ├── black_scholes.rs  # Core pricing engine
  └── lib.rs           # Module exports

crates/rust-py-bindings/src/
  ├── polars_greeks.rs # Polars plugin implementation  
  └── lib.rs          # PyO3 module setup

crates/rust-py-bindings/python/
  └── polars_greeks/   # Python package interface

crates/rust-py-bindings/tests/
  ├── test_greeks.py     # Integration tests
  └── test_performance.py # Benchmarks
```