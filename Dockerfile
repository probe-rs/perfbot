FROM rustlang/rust:nightly-bookworm-slim AS builder

RUN apt-get update && apt-get install -y libudev-dev pkg-config curl clang && rm -rf /var/lib/apt/lists/*

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --version 0.6.1 --force

WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
COPY assets/ ./assets/
COPY src/ ./src/
COPY Dioxus.toml .
COPY input.css .
COPY tailwind.config.js .

RUN ls -la /app

RUN dx build --release --platform web

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates

WORKDIR /app
COPY --from=builder /app/target/dx/perfbot/release/web/server ./server
COPY --from=builder /app/target/dx/perfbot/release/web/public ./public/

ENTRYPOINT ./server