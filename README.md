# Rust-Py Extension

A Rust workspace for creating Python extensions with multiple crates.

## Structure

- `crates/rust-py-core/` - Core Rust functionality
- `crates/rust-py-bindings/` - Python bindings using PyO3

## Build

```bash
cargo build --release
```

## Development

Add new crates to the workspace in root `Cargo.toml`.