# Multi-stage build for Communitas Bootstrap Node

# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Set working directory
WORKDIR /build

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release --bin bootstrap-node

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    openssl \
    curl

# Create non-root user
RUN addgroup -g 1000 communitas && \
    adduser -D -u 1000 -G communitas communitas

# Create necessary directories
RUN mkdir -p /data /config && \
    chown -R communitas:communitas /data /config

# Copy binary from build stage
COPY --from=builder /build/target/release/bootstrap-node /usr/local/bin/

# Copy configuration files
COPY config /config

# Switch to non-root user
USER communitas

# Expose ports
EXPOSE 7000/udp 7001/tcp

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:7001/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the bootstrap node
ENTRYPOINT ["/usr/local/bin/bootstrap-node"]
CMD ["--listen", "0.0.0.0:7000", "--metrics", "--metrics-port", "7001"]