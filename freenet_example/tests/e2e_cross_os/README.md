# E2E cross-OS tests

Here we test the final process cross-OS / mainnet.

- Requires `CROSS_OS_KEY` / `CROSS_OS_MACHINE`, 900 s timeout, `#[ignore]`.
- Both machines `cargo test --test cross_os_reconcile -- --ignored`.
- Tests `examples/connect_to_external.rs` (FREENET_HOST) and mainnet harness.
