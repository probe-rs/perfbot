# probe-rs perfbot

This is a webapplication that helps the probe-rs project track regressions and improvements.

It has a connection to our Matrix channel and can be controlled from there.
People can submit their benchmarks on their setups via the `benchmark.rs` in the main repo.

The webapp can be deployed by running

```bash
scripts/build_docker.sh
scripts/deploy.sh
```

During development, a `cargo run` suffices.

## Benchmarks

### RAM

When doing a RAM benchmark, please use a block size of 0x1000 Bytes and a Protocol speed of 1, 10 or 50MHz