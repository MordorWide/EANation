# Use the official Rust image for building the application
FROM rust:1.83-slim-bookworm AS builder

## Install the required build tools for OpenSSL
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential git \
    zlib1g-dev perl \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo manifest and lock file first to leverage Docker caching
COPY Cargo.toml Cargo.lock ./
COPY deps ./deps
COPY src ./src
COPY .cargo ./.cargo

# Compile the OpenSSL dependency
RUN ./deps/setup_deps.sh

# Build the actual application
RUN cargo build --release

# Use a lightweight image for running the application
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory for the runtime container
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/MordorWide /app/MordorWide

# Copy the entrypoint script
COPY docker/entrypoint.sh /app/entrypoint.sh

# Make the entrypoint script executable
RUN chmod +x /app/entrypoint.sh

# Expose the port your application listens on (change 8080 if necessary)
EXPOSE 18880 18885

# Use the entrypoint script as the container entry point
ENTRYPOINT ["/app/entrypoint.sh"]

# Set the default command for the container
CMD ["/app/MordorWide"]
