# Examples smoke tests

Here we import `examples/` and add tests for them.

- Each `tests/examples/<name>.rs` imports or spawns the corresponding `examples/<name>.rs` final process.
- Verifies the example builds and its crate logic (via `TestNode` or `cargo run --example`).
- Run: `cargo nextest run --test example_standalone_demo`.
