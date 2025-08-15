# Multi-stage build for P2P Bootstrap Node
FROM rust:1.75 as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy P2P project files
COPY ../../../p2p/Cargo.toml ./
COPY ../../../p2p/Cargo.lock ./
COPY ../../../p2p/crates ./crates/
COPY ../../../p2p/src ./src/

# Build the bootstrap node in release mode
RUN cargo build --release --bin saorsa

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /var/lib/saorsa saorsa

# Create directories
RUN mkdir -p /etc/saorsa /var/log/saorsa /var/lib/saorsa

# Copy binary from builder
COPY --from=builder /app/target/release/saorsa /usr/local/bin/saorsa

# Copy configuration files
COPY config/ /etc/saorsa/
COPY scripts/ /usr/local/bin/

# Set ownership
RUN chown -R saorsa:saorsa /var/lib/saorsa /var/log/saorsa /etc/saorsa

# Switch to app user
USER saorsa

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8888/health || exit 1

# Expose P2P port
EXPOSE 8888

# Start the bootstrap node
CMD ["/usr/local/bin/start-bootstrap.sh"]
EOF < /dev/null