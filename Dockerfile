FROM rust:1.87-slim as builder

# Install system dependencies required for building Rust applications with OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy the source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Use a smaller base image for the final image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/mmitm /usr/local/bin/mmitm

# Set the entrypoint
ENTRYPOINT ["mmitm"]

# Expose the port
EXPOSE 3000
