# syntax=docker/dockerfile:1.7
# Multi-stage build for the Astound Digital gateway.
# Stage 1 compiles the Rust workspace against the repo's .sqlx/ offline cache.
# Stage 2 ships a slim Debian runtime with the binaries + services/ YAML tree.
# The `artifacts` stage exports just the binaries, for the release tarball.
#
# Published by .github/workflows/release.yml on every push to main as
# ghcr.io/astound-digital/systemprompt-astound (docs/RELEASING.md). Build it
# locally with `just docker-build`.

FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    libssl-dev \
    pkg-config \
    clang \
    mold \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# rust-toolchain.toml pins a nightly; rustup installs it on first use. Copy the
# pin alone first so the toolchain download is its own cached layer and does
# not repeat on every source change.
COPY rust-toolchain.toml /src/rust-toolchain.toml
RUN rustup show

COPY . /src

ENV SQLX_OFFLINE=true \
    CC=clang \
    CXX=clang++

# Every binary the runtime's manifests (extensions/mcp/*/manifest.yaml and
# services/mcp/*.yaml) name must land in /out/bin — the MCP validator resolves
# binary -> manifest at boot and a missing one is a startup failure.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --workspace \
    && mkdir -p /out/bin \
    && cp target/release/systemprompt /out/bin/ \
    && cp target/release/systemprompt-mcp-agent /out/bin/

# Binaries only — `docker build --target artifacts --output type=local,dest=…`
# gives the release job its server tarball from the same compile.
FROM scratch AS artifacts
COPY --from=builder /out/bin /bin

# hey powers the demo/performance load tests; its upstream S3 binary host is
# dead (403), so build it from source and ship it on PATH — demo/_common.sh's
# install_hey() prefers a system hey.
FROM golang:1-bookworm AS heybuilder
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go install github.com/rakyll/hey@latest

FROM public.ecr.aws/aws-cli/aws-cli:2.36.18 AS awscli

FROM debian:bookworm-slim AS runtime

ARG SYSTEMPROMPT_VERSION=dev
LABEL org.opencontainers.image.title="Astound Digital gateway" \
      org.opencontainers.image.description="Astound Digital AI governance gateway — policy, audit, and MCP orchestration on systemprompt" \
      org.opencontainers.image.source="https://github.com/Astound-Digital/systemprompt-astound" \
      org.opencontainers.image.url="https://astounddigital.com" \
      org.opencontainers.image.documentation="https://github.com/Astound-Digital/systemprompt-astound/tree/main/docs" \
      org.opencontainers.image.vendor="Astound Digital" \
      org.opencontainers.image.version="${SYSTEMPROMPT_VERSION}"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    postgresql-client \
    lsof \
    jq \
    python3 \
    libreoffice-writer \
    libreoffice-calc \
    libreoffice-impress \
    fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

COPY --from=awscli /usr/local/aws-cli /usr/local/aws-cli
RUN ln -s /usr/local/aws-cli/v2/current/bin/aws /usr/local/bin/aws

RUN useradd -m -u 1000 app
WORKDIR /app

# /app/storage/data is runtime state (the gateway journal lives there since
# 0.58.1) and is excluded from the build context, so it must be created here:
# a named volume mounted at a path the image lacks comes up root-owned, and
# the server, running as uid 1000, cannot write it.
RUN mkdir -p /app/bin /app/logs /app/storage/data /app/web /app/.systemprompt/profiles/docker

COPY --from=builder /out/bin/ /app/bin/
COPY --from=heybuilder /go/bin/hey /app/bin/hey

COPY services /app/services
COPY storage /app/storage
COPY web /app/web
# The homepage demo showcase scans <system_root>/demo at runtime; without
# this the catalogue renders as an empty section.
COPY demo /app/demo
# MCP manifests live alongside their extension crates; the runtime validator
# globs extensions/mcp/*/manifest.yaml to resolve binary -> manifest.
COPY extensions/mcp /app/extensions/mcp

COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh /app/bin/* \
    && chown -R app:app /app

USER app
EXPOSE 8080

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:${PATH}" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_MCP_PATH=/app/bin \
    SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/docker/profile.yaml \
    WEB_DIR=/app/web

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

ENTRYPOINT ["/app/entrypoint.sh"]
