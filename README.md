# Smartknob

## Encoder sampling and display refresh

After display initialization, the application runs two concurrent async loops
within its main Embassy task. Encoder sampling targets a 1 ms period; display
refresh targets a 33 ms period. A single-slot Embassy signal retains the latest
position, replacing older unread samples. Encoder read errors are discarded
and retried; display errors still propagate to the application error handler.

Periods include the work done by each loop. If an iteration overruns its period,
the loop waits one period before trying again instead of issuing catch-up work.
These are scheduling targets, not real-time guarantees: synchronous drawing
still occupies the executor, and an in-flight SPI stripe still delays encoder
reads. Motor control is not implemented yet.

Each display frame is sent as 30 stripes of 8 rows (3840 pixel bytes per stripe).
Each stripe sets its own display window. After writing its pixels, the SPI mutex
is released and the display loop yields so encoder sampling can run. At the
configured 80 MHz, pixel transmission takes about 384 us per stripe, excluding
commands, software overhead and encoder reads. The full framebuffer is retained;
this change reduces uninterrupted bus occupancy, not total pixel traffic.

On hardware, verify that the position indicator follows rotation and that SPI
reads occur during a display refresh, between stripes. With a logic analyzer,
check that display CS and encoder CS never overlap, that pixel transfers contain
3840 bytes, and that encoder transactions use their configured 1 MHz clock.
Check all screen rows for missing or shifted pixels and measure the maximum
encoder interval while rotating continuously. The 1 ms sampling period is a
target to validate on the board, not a guaranteed upper bound.

The MT6701 uses SPI mode 1 at 1 MHz. Each read clocks the complete 24-bit SSI
frame: 14 angle bits, 4 status bits and 6 CRC bits. The decoder checks the CRC
before interpreting status, and rejects loss of tracking, strong/weak magnetic
fields and the reserved field status. Push detection is valid and retained in
`Position.status` bit 2. The sampling loop discards invalid frames and SPI read
errors, preserving the last valid position and logging at most one warning per
second. After a failure lasting at least 500 ms, recovery is logged when valid
readings resume. The screen retains the last valid number and indicator; errors are
reported only in the logs. Before the first valid reading, the number is zero.
The next valid reading updates the display and logs recovery. Invalid positions
are never published as valid samples. The indicator's zero is at the top of the
screen, at framebuffer coordinates (120, 15).

The protocol follows the manufacturer's [MT6701 datasheet, SSI Read Angle](https://www.magntek.com.cn/upload/pdf/202407/MT6701_Rev.1.8.pdf).
Hardware validation should check 24 clock pulses per read, falling-edge data
capture, and matching CRCs on captured frames.

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
