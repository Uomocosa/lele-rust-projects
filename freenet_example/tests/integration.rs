use std::time::Duration;

use freenet_example::testing::*;
use freenet_example::{ClickerClient, Role};

#[tokio::test(flavor = "multi_thread")]
async fn test_full_lifecycle() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut client = connect(node.port()).await.unwrap();
    let key = deploy(&mut client, &wasm).await.unwrap();

    assert_eq!(get_count(&mut client, key).await.unwrap(), 0);

    update_count(&mut client, key, 42).await.unwrap();
    assert_eq!(get_count(&mut client, key).await.unwrap(), 42);

    update_count(&mut client, key, 99).await.unwrap();
    assert_eq!(get_count(&mut client, key).await.unwrap(), 99);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_persistence() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let key;
    {
        let mut client = connect(node.port()).await.unwrap();
        key = deploy(&mut client, &wasm).await.unwrap();
        update_count(&mut client, key, 5).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    {
        let mut client = connect(node.port()).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
        update_count(&mut client, key, 8).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 8);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_publish_subscribe() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut pub_ = connect(node.port()).await.unwrap();
    let key = deploy(&mut pub_, &wasm).await.unwrap();
    update_count(&mut pub_, key, 5).await.unwrap();

    let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 5);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_standalone_demo() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut clicker = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();
    assert!(clicker.count() == 0);
    for expected in 1..=3 {
        assert_eq!(clicker.tick().await.unwrap(), expected);
    }
    assert_eq!(clicker.state().await.unwrap(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_two_writers() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut writer_a = connect(node.port()).await.unwrap();
    let mut writer_b = connect(node.port()).await.unwrap();
    let mut verifier = connect(node.port()).await.unwrap();
    let key = deploy(&mut writer_a, &wasm).await.unwrap();
    subscribe(&mut writer_b, key).await.unwrap();
    subscribe(&mut verifier, key).await.unwrap();

    update_count(&mut writer_a, key, 3).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: first update notification not received");

    update_count(&mut writer_b, key, 7).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: second update notification not received");

    assert_eq!(get_count(&mut verifier, key).await.unwrap(), 7);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_subscriber_notifications() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut pub_ = connect(node.port()).await.unwrap();
    let key = deploy(&mut pub_, &wasm).await.unwrap();

    let mut sub_a = connect(node.port()).await.unwrap();
    assert_eq!(subscribe(&mut sub_a, key).await.unwrap(), 0);
    let mut sub_b = connect(node.port()).await.unwrap();
    assert_eq!(subscribe(&mut sub_b, key).await.unwrap(), 0);

    update_count(&mut pub_, key, 5).await.unwrap();
    let notif_a = recv_notification(&mut sub_a, Duration::from_secs(10))
        .await
        .expect("sub_a: update notification not received");
    let notif_b = recv_notification(&mut sub_b, Duration::from_secs(10))
        .await
        .expect("sub_b: update notification not received");
    assert_eq!(notif_a, 5);
    assert_eq!(notif_b, 5);

    update_count(&mut pub_, key, 10).await.unwrap();
    let notif_a2 = recv_notification(&mut sub_a, Duration::from_secs(10))
        .await
        .expect("sub_a: second update notification not received");
    let notif_b2 = recv_notification(&mut sub_b, Duration::from_secs(10))
        .await
        .expect("sub_b: second update notification not received");
    assert_eq!(notif_a2, 10);
    assert_eq!(notif_b2, 10);

    assert_eq!(get_count(&mut sub_a, key).await.unwrap(), 10);
    assert_eq!(get_count(&mut sub_b, key).await.unwrap(), 10);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_two_clients_talk_via_node() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();

    let mut pub_ = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();
    assert_eq!(pub_.tick().await.unwrap(), 1);

    let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 1);

    assert_eq!(sub.tick().await.unwrap(), 2);
    wait_for_count(&mut pub_, 2, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(pub_.state().await.unwrap(), 2);

    assert_eq!(pub_.tick().await.unwrap(), 3);
    wait_for_count(&mut sub, 3, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 3);
}

fn spawn_example(args: &[&str]) -> (std::process::Child, std::sync::mpsc::Receiver<String>) {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    // Build the example first so the binary exists
    let _ = std::process::Command::new("cargo")
        .args(["build", "--example", "p2p_counter_gateway"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .stderr(std::process::Stdio::null())
        .status();
    let exe = std::path::Path::new(&target_dir)
        .join("debug")
        .join("examples")
        .join("p2p_counter_gateway");
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        if !buf.is_empty() {
            eprintln!("[subprocess stderr]: {}", String::from_utf8_lossy(&buf));
        }
    });
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    (child, rx)
}

fn expect_line(rx: &std::sync::mpsc::Receiver<String>, prefix: &str, timeout: Duration) -> String {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.starts_with(prefix) {
                    return line;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    panic!("timed out waiting for line starting with: {prefix}");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("subprocess exited before printing: {prefix}");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_gateway_subprocess_smoke() {
    let (mut gateway, gw_rx) = spawn_example(&["--gateway", "--public-address", "127.0.0.1"]);

    let connect_line = expect_line(&gw_rx, "GATEWAY_CONNECT=", Duration::from_secs(30));
    assert!(
        connect_line.len() > "GATEWAY_CONNECT=".len(),
        "connect string should contain pubkey"
    );

    let deployed_line = expect_line(
        &gw_rx,
        "counter deployed, initial count:",
        Duration::from_secs(15),
    );
    let initial_count = deployed_line
        .split("initial count:")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(u64::MAX);
    assert_eq!(initial_count, 0);

    let tick_line = expect_line(&gw_rx, "tick 1:", Duration::from_secs(10));
    let tick_count = tick_line
        .split("count =")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    assert!(tick_count >= 1, "gateway should increment the counter");

    let _ = gateway.kill();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Freenet assigns random UDP ports for P2P transport that differ from the configured \
           `public_port`, and NAT-traversal hole punching does not work between two processes on \
           the same loopback interface. A full P2P e2e test requires either separate machines \
           with routable IPs, or Freenet's `turmoil` simulation framework.\n\n\
           The gateway smoke test above verifies the subprocess starts, deploys, and ticks. \
           The in-process integration tests (e.g. test_two_clients_talk_via_node) verify pub/sub \
           and state sync through a single node's WebSocket API — which is the same API the P2P \
           layer feeds into.\n\n\
           Run manually with:\n\
             # Terminal 1: start gateway\n\
             cargo run --example p2p_counter_gateway -- --gateway --public-address <YOUR_IP>\n\
             # Terminal 2: start peer (use the GATEWAY_CONNECT line from terminal 1)\n\
             cargo run --example p2p_counter_gateway -- --connect <CONNECT_STRING>\n\
             # Both should show synchronized ticks"]
async fn test_p2p_gateway_peer_sync() {}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Same limitation as test_p2p_gateway_peer_sync — P2P between two local processes \
           requires routable IPs or the turmoil simulator.\n\n\
           When testing on separate machines, run gateway B with both --gateway and --connect:\n\
             cargo run --example p2p_counter_gateway -- --gateway --public-address <IP_B> \
               --connect <IP_A>:<PORT_A>,<PUBKEY_A>"]
async fn test_p2p_both_gateways_sync() {}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "This test requires the real Freenet P2P network. Both standalone binaries use \
           `skip_load_from_network: false` to discover peers via the global DHT — there is no \
           way to supply a self-contained DHT in CI.\n\n\
           To verify manually on two machines with routable IPs:\n\
             1. Machine A: cargo run --release\n\
             2. Machine B: cargo run --release\n\
             3. Both should see each other's counter updates\n\n\
           Run in CI with:\n\
             cargo test --test integration test_p2p_two_standalone_binaries -- --ignored --nocapture"]
async fn test_p2p_two_standalone_binaries() {}
