# syntax=docker/dockerfile:1.7

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin crab-antiagiota

FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN useradd --create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/crab-antiagiota /usr/local/bin/crab-antiagiota
COPY resources/references.json resources/mcc-risk.json resources/normalization.json /app/resources/

USER appuser
EXPOSE 9999

ENTRYPOINT ["/usr/local/bin/crab-antiagiota"]
