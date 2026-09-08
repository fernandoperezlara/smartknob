# Smartknob

Rust `no_std` firmware for the ESP32-C6, with an SPI display and encoder.

## Development

The firmware pins Rust 1.98.1 and the `riscv32imac-unknown-none-elf` target in
`firmware/rust-toolchain.toml`. Rustup installs the required components when
running Cargo from that directory.

```sh
cd firmware
cargo build --release --locked
cargo clippy --release --locked -- -D warnings
```

Formatting uses a separate nightly for the existing unstable rustfmt
options; firmware compilation uses stable Rust.

```sh
rustup toolchain install nightly --profile minimal --component rustfmt
cargo +nightly fmt --all -- --check
```

With `just` installed, the equivalent commands are `just build`, `just check`,
and `just fmt check`. With `espflash` installed and the board connected,
`cargo run --release --locked` builds, flashes, and opens the serial monitor.

All firmware dependencies come from crates.io releases. Commit `Cargo.lock`
with dependency updates and use `--locked` for validation. The HAL's `unstable`
feature is still needed for DMA APIs; its minor version is constrained to 1.2
so changing that API requires an explicit manifest update.

The Docker Compose build uses the same firmware toolchain:

```sh
docker compose build build
docker compose run --rm build
```
