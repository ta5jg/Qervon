# ==============================================================================
# File:           Makefile
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-05
# Version:        0.1.0
#
# Description:
#   Direct local development commands. Docker is optional and only runs dependencies.
#
# Specification:
#   QMI-000000 and applicable Qervon specifications.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

.DEFAULT_GOAL := help

.PHONY: help api worker test test-postgres check fmt clippy migrate release-build production-preflight test-deploy dev-services-up dev-services-down

help:
	@printf '%s\n' 'make api              Run the API directly on this computer' \
		'make worker           Run the durable webhook outbox worker' \
		'make test             Run backend tests' \
		'make test-postgres    Run opt-in real PostgreSQL repository tests' \
		'make check            Run format, Clippy and tests' \
		'make migrate          Apply PostgreSQL migrations' \
		'make release-build    Build direct production binaries' \
		'make production-preflight Validate a production environment file' \
		'make test-deploy      Test production deployment safeguards' \
		'make dev-services-up  Start optional local PostgreSQL and Redis' \
		'make dev-services-down Stop optional local PostgreSQL and Redis'

api:
	cd backend && cargo run -p qervon-api-gateway

worker:
	cd backend && cargo run -p qervon-worker

test:
	cd backend && cargo test --workspace

test-postgres:
	bash scripts/test-postgres-integration.sh

fmt:
	cd backend && cargo fmt --all -- --check

clippy:
	cd backend && cargo clippy --workspace --all-targets -- -D warnings

check: fmt clippy test

migrate:
	cd backend && cargo run -p qervon-migration-runner

release-build:
	scripts/build-release.sh

production-preflight:
	scripts/production-preflight.sh

test-deploy:
	scripts/test-production-preflight.sh

dev-services-up:
	docker compose --env-file .env.local up -d

dev-services-down:
	docker compose down
