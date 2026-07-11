FROM rust:1.78-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/harness /usr/local/bin/harness
WORKDIR /workspace
ENTRYPOINT ["harness"]