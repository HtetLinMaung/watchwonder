# Use the official Rust image as the builder
FROM rust:slim-buster as builder

# Set the current working directory inside the container
WORKDIR /usr/src/app

# Install OpenSSL development packages
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy over your manifest
COPY Cargo.toml Cargo.lock ./

# Cache dependencies - this will only re-run if your manifest files change
# RUN cargo fetch

# Copy your source code
COPY src ./src

# Build the application
RUN cargo build --release

# For the final stage, use a small image
FROM debian:12-slim

# Install necessary libraries. This might change based on your application's requirements
RUN apt-get update && apt-get install -y libpq5 ibssl-dev && rm -rf /var/lib/apt/lists/*

# Copy over the built binary file from the builder stage
COPY --from=builder /usr/src/app/target/release/watchwonder /usr/local/bin/

COPY images /images

# Set the start command
CMD ["watchwonder"]