FROM rust:1-slim AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
# Compile dependencies separately for layer caching
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -f target/release/deps/pqno*

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/pqno ./pqno
COPY static ./static

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["./pqno"]
