FROM rust:1.86.0 AS builder
WORKDIR /app

# Cache dependencies
# COPY Cargo.toml Cargo.lock ./
# RUN cargo fetch

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# --- Runtime Stage ---
# Bookworm has OpenSSL 3.x. Use Ubuntu 22.04+ or Debian Bookworm+.
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    libsqlite3-0 \
  && rm -rf /var/lib/apt/lists/*

# Create a non-root user (optional but recommended)
RUN useradd -m appuser

WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/wikimapia-backend .

# # Use the non-root user
USER appuser

# Expose the port your app listens on (change if needed)
EXPOSE 3333

# Run the binary
CMD ["./wikimapia-backend", "server"]