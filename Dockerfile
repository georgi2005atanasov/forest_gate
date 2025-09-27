# --- build stage ---
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Use offline metadata produced by `cargo sqlx prepare`
ENV SQLX_OFFLINE=true
ARG BIN_NAME=forest_gate   # <— use your actual bin

# Prime dependency cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release && \
    rm -rf target/release/deps/* src/main.rs

# Copy source AND the sqlx cache
COPY .sqlx ./.sqlx
COPY src ./src

# Build the actual binary
RUN cargo build --release --bin ${BIN_NAME}

# --- runtime stage ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy the single binary to a new path
COPY --from=builder /app/target/release/forest_gate /usr/local/bin/app
RUN chmod +x /usr/local/bin/app

COPY scripts /app/scripts

ENV RUST_LOG=info
EXPOSE 8080
CMD ["app"]