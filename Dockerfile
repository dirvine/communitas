# Communitas - Production Docker Image
# Enhanced security and production-ready configuration

FROM rust:1.85-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-dev \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application in release mode with security optimizations
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always
RUN cargo build --release --locked

# Production runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-37 \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r communitas && useradd -r -g communitas communitas

# Set working directory
WORKDIR /app

# Copy the built application
COPY --from=builder /app/target/release/communitas /app/communitas

# Create data directory with proper permissions
RUN mkdir -p /app/data && chown -R communitas:communitas /app/data

# Switch to non-root user
USER communitas

# Set environment variables for production
ENV RUST_LOG=info,communitas=info,saorsa_core=warn
ENV COMMUNITAS_DATA_DIR=/app/data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:1420/health || exit 1

# Expose port (if needed for web interface)
EXPOSE 1420

# Run the application
CMD ["./communitas"]