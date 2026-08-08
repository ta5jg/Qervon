# =============================================================================
# File:           Dockerfile
# Project:        Qervon Logistics Operating System (LOS)
# Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
# Description:    Multi-stage Dockerfile for Qervon Rust API Gateway & Static Apps
# =============================================================================

FROM rust:1.78-slim as builder

WORKDIR /usr/src/qervon
COPY . .

RUN cargo build --release -p qervon-api-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/qervon/target/release/qervon-api-gateway /app/qervon-api-gateway
COPY --from=builder /usr/src/qervon/backend/apps/api-gateway/static /app/backend/apps/api-gateway/static

EXPOSE 8080
ENV RUST_LOG=info

CMD ["/app/qervon-api-gateway"]
