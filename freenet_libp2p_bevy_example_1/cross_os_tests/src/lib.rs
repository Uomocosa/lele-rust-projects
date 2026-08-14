//! Cross-OS test crate. Only ever run explicitly via the self-hosted workflow's `cross-os`
//! job (`cargo test -- --ignored`) on both the Linux and Windows runners. The single test in
//! `tests/cross_os_sync.rs` is `#[ignore]`d by default and exercises the public-mainnet sync
//! path, so the two machines need not be on the same network.

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        // lib is a placeholder; the real coverage is the ignored integration test in tests/
    }
}
