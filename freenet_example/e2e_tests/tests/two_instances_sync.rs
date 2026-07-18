use e2e_tests::*;
use std::time::Duration;

#[test]
fn test_two_instances_sync() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let binary = build_release_binary();

        let (mut a, rx_a) = spawn_binary(&binary, &["--p2p-port", "41338"]);
        let connected_a =
            expect_line(&rx_a, "connected, running indefinitely", Duration::from_secs(120));

        let parse_count = |line: &str| -> u64 {
            line.split("count=")
                .nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0)
        };
        let a_count = parse_count(&connected_a);

        let first_tick_a = expect_line(&rx_a, "tick", Duration::from_secs(30));
        let a_tick = parse_count(&first_tick_a);
        assert!(a_tick > a_count, "instance A should increment");

        let (mut b, rx_b) = spawn_binary(&binary, &["--p2p-port", "41339"]);
        let connected_b =
            expect_line(&rx_b, "connected, running indefinitely", Duration::from_secs(120));
        let b_count = parse_count(&connected_b);
        assert!(
            b_count >= a_count,
            "instance B should see at least A's initial count ({a_count}), got {b_count}"
        );

        let _ = a.kill();
        let _ = b.kill();
    });
}
