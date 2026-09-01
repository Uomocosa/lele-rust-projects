use std::time::Duration;

/// Three instances: two converge first, then a third joins mid-session.
/// All three must converge on identical state hashes.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires live freenet mainnet — run via /run-local-mainnet ex4"]
async fn three_node_live_join_convergence() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing_4::unique_params();
    let wasm = testing_4::load_wasm();

    let mut host = testing_4::ProductionGameApp::new_local(&wasm, &params, 0, None).await;
    let gateway = host.freenet_gateway();

    let mut a =
        testing_4::ProductionGameApp::new_local(&wasm, &params, 1, Some(gateway.clone())).await;

    host.wait_for_roster_len(2, Duration::from_secs(120))
        .await
        .map_err(|e| format!("host roster: {e}"))?;
    a.wait_for_roster_len(2, Duration::from_secs(120))
        .await
        .map_err(|e| format!("a roster: {e}"))?;

    let host_hash = host.state_hash();
    let a_hash = a.state_hash();
    assert_ne!(host_hash, 0, "host must have a state hash");
    assert_ne!(a_hash, 0, "a must have a state hash");

    let mut b = testing_4::ProductionGameApp::new_local(&wasm, &params, 2, Some(gateway)).await;

    host.wait_for_roster_len(3, Duration::from_secs(120))
        .await
        .map_err(|e| format!("host roster after B join: {e}"))?;
    a.wait_for_roster_len(3, Duration::from_secs(120))
        .await
        .map_err(|e| format!("a roster after B join: {e}"))?;
    b.wait_for_roster_len(3, Duration::from_secs(120))
        .await
        .map_err(|e| format!("b roster: {e}"))?;

    tokio::time::sleep(Duration::from_secs(30)).await;

    let host_hash = host.state_hash();
    let a_hash = a.state_hash();
    let b_hash = b.state_hash();
    assert_ne!(
        host_hash, 0,
        "host must have a state hash after convergence"
    );
    assert_ne!(a_hash, 0, "a must have a state hash after convergence");
    assert_ne!(b_hash, 0, "b must have a state hash after convergence");
    assert_eq!(host_hash, a_hash, "host and a must converge");
    assert_eq!(a_hash, b_hash, "a and b must converge");

    drop(host);
    drop(a);
    drop(b);
    Ok(())
}
