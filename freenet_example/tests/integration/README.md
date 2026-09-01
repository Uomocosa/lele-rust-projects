# Integration tests

Here we test the crate (library and `main` via code, in-process).

- Use `TestNode::start()` / `NodeConfig` in-process, no `Command` or `cargo run --example`.
- Fast, deterministic, no network, runs with `cargo nextest run --test full_lifecycle`.
- Example: `tests/integration/standalone_demo.rs` mirrors `examples/standalone_demo.rs` but asserts.
