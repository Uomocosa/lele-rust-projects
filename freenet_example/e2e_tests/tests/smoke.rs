use e2e_tests::*;
use std::time::Duration;

#[test]
fn test_smoke() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let binary = build_release_binary();
        let (mut child, rx) = spawn_binary(&binary, &[]);

        let connected =
            expect_line(&rx, "connected, running indefinitely", Duration::from_secs(60));
        assert!(connected.contains("key="), "should print contract key");
        assert!(connected.contains("count="), "should print initial count");

        let first_tick = expect_line(&rx, "tick", Duration::from_secs(30));
        let second_tick = expect_line(&rx, "tick", Duration::from_secs(30));

        let parse_count = |line: &str| -> u64 {
            line.split("count=")
                .nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0)
        };
        let c1 = parse_count(&first_tick);
        let c2 = parse_count(&second_tick);
        assert!(c2 > c1, "tick count should increment: {c1} -> {c2}");

        let _ = child.kill();
    });
}
