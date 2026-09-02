use std::time::Duration;

use freenet_example::testing::{expect_line, spawn_example};

#[tokio::test(flavor = "multi_thread")]
async fn test_gateway_subprocess_smoke() {
    let (mut gateway, gw_rx) = spawn_example(
        "p2p_counter_gateway",
        &["--gateway", "--public-address", "127.0.0.1"],
    )
    .expect("spawn example");

    let connect_line =
        expect_line(&gw_rx, "GATEWAY_CONNECT=", Duration::from_secs(30)).expect("connect line");
    assert!(
        connect_line.len() > "GATEWAY_CONNECT=".len(),
        "connect string should contain pubkey"
    );

    let deployed_line = expect_line(
        &gw_rx,
        "counter deployed, initial count:",
        Duration::from_secs(15),
    )
    .expect("deployed line");
    let initial_count = deployed_line
        .split("initial count:")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(u64::MAX);
    assert_eq!(initial_count, 0);

    let tick_line = expect_line(&gw_rx, "tick 1:", Duration::from_secs(10)).expect("tick line");
    let tick_count = tick_line
        .split("count =")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    assert!(tick_count >= 1, "gateway should increment the counter");

    let _ = gateway.kill();
}
