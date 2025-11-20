# Use Debian slim as base (the binary is already built)
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 antsol

# Create app directory
WORKDIR /app

# Copy pre-built binary from host
COPY target/release/antsol-indexer-v2 /app/antsol-indexer-v2

# Set ownership
RUN chown -R antsol:antsol /app

# Switch to non-root user
USER antsol

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["/app/antsol-indexer-v2"]
