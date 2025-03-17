# Build stage
FROM rust:1.73-slim-bullseye as builder

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*
z
# Copy dependency specifications
COPY Cargo.toml ./
COPY ../Cargo.lock ./

# Create dummy source files to cache dependencies
RUN mkdir -p src/resources && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release

# Copy actual source code
COPY src ./src
COPY resources ./resources

# Touch main.rs to force rebuild
RUN touch src/main.rs

# Build actual application
RUN cargo build --release --bin messier

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binary from builder
COPY --from=builder /app/target/release/messier /app/messier

# Copy configuration files
COPY --from=builder /app/resources /app/resources

# Set environment variables for config-rs
ENV CONFIG_PATH=/app/resources/application.yml
ENV RUST_LOG=info

# Expose the port your Axum application listens on
EXPOSE 3004

# Run the application
ENTRYPOINT ["/app/messier"]