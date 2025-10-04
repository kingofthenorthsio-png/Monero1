# monero-oxide

A modern Monero transaction library. It provides a modern, Rust-friendly view of
the Monero protocol.

This library is usable under no-std when the `std` feature (on by default) is
disabled.

Recommended usage of the library is with `overflow-checks = true`, even for
release builds.

### Cargo Features

- `std` (on by default): Enables `std` (and with it, more efficient internal
  implementations).
- `compile-time-generators` (on by default): Derives the generators at
  compile-time so they don't need to be derived at runtime. This is recommended
  if program size doesn't need to be kept minimal.
- `multisig`: Enables the `multisig` feature for all dependencies.
