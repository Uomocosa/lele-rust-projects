# E2E local tests

Here we test the final process on one OS (via command line).

- Spawn the built binary via `cargo run --example` or `Command::new("cargo")`.
- Tests the `examples/` final process, e.g. `p2p_counter_gateway` gateway `--gateway --connect`.
- Run: `cargo nextest run --test gateway_subprocess_smoke`.
