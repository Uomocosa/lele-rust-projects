use e2e_tests::*;
use std::time::Duration;

#[test]
fn test_ws_bridge_sync() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let target_dir =
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "../target".into());
        let exe = std::path::Path::new(&target_dir)
            .join("release")
            .join("examples")
            .join("p2p_counter_ws_bridge");
        assert!(
            exe.exists(),
            "Example binary not found at {} — run `cargo build --release --examples` first",
            exe.display()
        );

        let (mut host, rx_host) = spawn_binary(&exe.to_string_lossy(), &[]);

        let host_line =
            expect_line(&rx_host, "Host node listening on", Duration::from_secs(30));
        let host_port = host_line
            .rsplit(':')
            .next()
            .and_then(|s| s.trim().parse::<u16>().ok())
            .expect("failed to parse host port");

        let connect_arg = format!("127.0.0.1:{host_port}");
        let (mut client, rx_client) =
            spawn_binary(&exe.to_string_lossy(), &["--connect", &connect_arg]);
        let client_connected =
            expect_line(&rx_client, "connected, counter state:", Duration::from_secs(30));
        assert!(
            client_connected.contains("counter state:"),
            "client should report counter state"
        );

        let _ = host.kill();
        let _ = client.kill();
    });
}
