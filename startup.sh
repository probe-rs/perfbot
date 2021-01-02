#!/bin/sh

echo "Starting up webserver"

ls /app/data

RUST_BACKTRACE=1 \
/app/perfbot