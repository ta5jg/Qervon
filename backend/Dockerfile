# ==============================================================================
# File:           backend/Dockerfile
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-05
# Version:        0.1.0
#
# Description:
#   Multi-stage Dockerfile for Qervon API Gateway production build.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

# Build stage
FROM rust:1.90-slim as builder

WORKDIR /app

# Copy workspace manifests and source
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY crates ./crates
COPY modules ./modules
COPY foundation ./foundation
COPY migrations ./migrations

# Build production release binary
RUN cargo build --release -p qervon-api-gateway

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/qervon-api-gateway /app/qervon-api-gateway

ENV RUST_LOG=info
ENV QERVON_STORAGE=memory
EXPOSE 3000

CMD ["/app/qervon-api-gateway"]
