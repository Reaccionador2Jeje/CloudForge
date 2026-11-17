# Build CloudForge API Gateway in a reproducible Linux container.
FROM rust:1.85-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release -p cloudforge-api-gateway

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --create-home cloudforge
WORKDIR /app
COPY --from=builder /workspace/target/release/cloudforge-api-gateway /usr/local/bin/cloudforge-api-gateway
USER cloudforge
ENV CLOUDFORGE_HOST=0.0.0.0 CLOUDFORGE_PORT=8080 RUST_LOG=info
EXPOSE 8080
ENTRYPOINT ["cloudforge-api-gateway"]
