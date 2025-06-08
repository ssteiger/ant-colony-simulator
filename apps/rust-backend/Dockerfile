# Build stage
FROM rust:1.70-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy" > src/lib.rs

# Build dependencies
RUN cargo build --release && \
    rm -rf src target/release/deps/ant_colony_simulator* target/release/simulator*

# Copy actual source code
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false simulator

# Copy binary from builder stage
COPY --from=builder /app/target/release/simulator /usr/local/bin/simulator

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/simulator

# Switch to non-root user
USER simulator

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep simulator || exit 1

# Set default command
CMD ["simulator"] 