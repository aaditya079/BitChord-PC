# Contributing to BitChord PC

Thank you for helping improve **BitChord PC**!

## Prerequisites

- **Rust** (stable toolchain, edition 2024) via [rustup.rs](https://rustup.rs)
- **Windows:** Visual Studio Build Tools (MSVC) or MinGW-w64 UCRT
- **Linux (Debian/Ubuntu):** `libasound2-dev`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libdbus-1-dev`, `libssl-dev`, `pkg-config`

## Building & Testing

```bash
# Check compilation & lints (enforced in CI)
cargo clippy --locked --all-targets -- -D warnings

# Run the unit test suite
cargo test --locked

# Build optimized release binary
cargo build --release --locked
```

## Pull Request Workflow

1. Branch from `main` using a descriptive name (`feat/...`, `fix/...`, `docs/...`).
2. Keep PRs focused on a single feature or bug fix.
3. Ensure `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass before opening your PR.
4. For hardware/driver changes (e.g., `src/lighting.rs`), ensure all hardware I/O is strictly gated and unit-tested against pure logic or mocks so CI never touches physical devices.
