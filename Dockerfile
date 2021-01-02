# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.48-alpine as cargo-build


# RUN sudo apt-get update && \
#     sudo apt-get install -y \
#     libstdc++-8-dev \
#     libc-dev \
#     build-essential

RUN apk \
    --update \
    --upgrade \
    --no-cache \
    add \
    build-base \
    openssl-dev \
    cmake

# Copy the source
COPY src/ src/
COPY Cargo.toml Cargo.toml
COPY templates/ templates/
COPY migrations/ migrations/

ENV RUSTFLAGS -Ctarget-feature=-crt-static

# Build the application.
RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN apk \
    --update \
    --upgrade \
    --no-cache \
    add \
    libstdc++

WORKDIR /app

RUN mkdir housekeeping
RUN mkdir data

COPY --from=cargo-build /usr/local/cargo/bin/perfbot /app/perfbot

COPY startup.sh startup.sh
COPY templates/ templates/
COPY static/ static/

RUN chmod 777 startup.sh

EXPOSE 3333

CMD /app/startup.sh