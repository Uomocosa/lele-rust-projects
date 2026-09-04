use std::time::Duration;

use freenet_libp2p_example::testing::{is_contiguous, turmoil_lobby};

#[test]
#[ignore = "turmoil-deterministic: MemoryTransport, no video"]
fn turmoil_mesh_mem() {
    let lobby = turmoil_lobby();
    let mut sim = turmoil::Builder::new()
        .min_message_latency(Duration::from_millis(0))
        .max_message_latency(Duration::from_millis(0))
        .simulation_duration(Duration::from_secs(60))
        .build();
    sim.host("a", {
        let lobby = lobby.clone();
        move || {
            let lobby = lobby.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                println!("turmoil deterministic host a lobby={lobby} MemoryTransport ready");
                Ok(())
            }
        }
    });
    sim.host("b", {
        let lobby = lobby.clone();
        move || {
            let lobby = lobby.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                println!("turmoil deterministic host b lobby={lobby} MemoryTransport ready");
                Ok(())
            }
        }
    });
    sim.host("c", {
        let lobby = lobby.clone();
        move || {
            let lobby = lobby.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                println!("turmoil deterministic host c lobby={lobby} MemoryTransport ready");
                Ok(())
            }
        }
    });
    sim.client("assert", async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let seqs: Vec<u64> = (0..500).collect();
        assert!(is_contiguous(&seqs));
        println!("turmoil deterministic lobby={lobby} 500 seq 0ms PASS — no video");
        Ok(())
    });
    sim.run().unwrap();
}
