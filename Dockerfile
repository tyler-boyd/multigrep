FROM rust
WORKDIR /multigrep
RUN USER=root cargo init --bin .
COPY Cargo.lock Cargo.toml ./
RUN cargo build --release
RUN rm -rf src/ target/release/deps/multigrep

COPY src/ src/
RUN cargo build --release
RUN cargo install --path .
