##########################################
# Rust Environment
##########################################
FROM rust:1.98.1

RUN rustup component add rustfmt clippy
RUN rustup target add riscv32imac-unknown-none-elf
RUN rustup toolchain install nightly --profile minimal --component rustfmt
RUN cargo install just --locked

# Set working directory
WORKDIR /app

# Copy Rust configuration files
COPY rust-toolchain.toml rustfmt.toml ./
COPY justfile ./
COPY .just/ ./.just/

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN mkdir -p build && echo "fn main() {}" > build/main.rs
COPY .cargo/ ./.cargo/
RUN cargo fetch --locked
RUN rm -rf src/ build/

# Copy project files
COPY src/ ./src/
COPY build/ ./build/
COPY assets/ ./assets/

# Default command
CMD ["just", "check"]
