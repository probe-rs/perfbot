# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM ekidd/rust-musl-builder:1.48.0 as cargo-build


RUN sudo apt-get update && \
    sudo apt-get install -y \
    libstdc++-8-dev \
    libc-dev \
    build-essential

# Copy the source
COPY src/ src/
COPY Cargo.toml Cargo.toml

ENV RUSTFLAGS -Ctarget-feature=-crt-static

# Build the application.
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

WORKDIR /app

RUN mkdir housekeeping
RUN mkdir data

COPY --from=cargo-build /home/rust/src/target/x86_64-unknown-linux-musl/release/perfbot .

COPY startup.sh startup.sh
COPY templates/ data/templates/
COPY static/ data/static/

RUN chmod 777 startup.sh

EXPOSE 3333

CMD /app/startup.sh