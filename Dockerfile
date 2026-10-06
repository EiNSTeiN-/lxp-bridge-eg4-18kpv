# syntax=docker/dockerfile:1
FROM rust:1.90-bookworm AS builder
WORKDIR /usr/src/lxp-bridge
COPY Cargo.toml Cargo.lock build.rs ./
COPY .cargo .cargo
COPY src src
COPY db db
RUN cargo build --locked --release

FROM builder AS test
COPY tests tests
COPY config.yaml.example ./
RUN cargo test --locked --features=mocks

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/lxp-bridge/target/release/lxp-bridge /usr/local/bin/lxp-bridge
ENTRYPOINT ["lxp-bridge", "-c", "/etc/config.yaml"]
