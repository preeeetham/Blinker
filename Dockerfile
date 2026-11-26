# syntax=docker/dockerfile:1
# Multi-stage build: compile server + WASM bundle, ship a ~100 MB Debian image
# running a single static-ish binary as an unprivileged user.

FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
 && cargo install cargo-leptos --locked
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target/cache \
    cargo leptos build --release \
 && mkdir -p /out && cp target/release/blinker-web /out/blinker && cp -r target/site /out/site

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates tini \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home /app blinker \
 && mkdir -p /data && chown blinker /data
WORKDIR /app
COPY --from=build /out/ /app/
ENV LEPTOS_OUTPUT_NAME=blinker \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    DATA_DIR=/data \
    RUST_LOG=info
USER blinker
VOLUME ["/data"]
EXPOSE 3000
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/blinker"]
