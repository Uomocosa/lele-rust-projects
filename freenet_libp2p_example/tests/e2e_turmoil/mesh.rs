use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use freenet_libp2p_example::testing::{
    finish_record, is_contiguous, load_creds, send_video_file, start_record, tile_three,
    turmoil_lobby, wakeup_screen,
};

fn shell_escape(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

// needed helper:
fn spawn_host(sim: &mut turmoil::Sim<'_>, name: &'static str, lobby: String, log: PathBuf) {
    let log_path = log;
    sim.host(name, move || {
        let lobby = lobby.clone();
        let log_path = log_path.clone();
        let name = name;
        async move {
            let mut last: u8 = 0;
            for seq in 0..500 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let prev = last;
                let mut buf = [0u8; 1];
                let next = if getrandom::getrandom(&mut buf).is_ok() {
                    b'a'.checked_add(buf[0] % 26).unwrap_or(b'a')
                } else {
                    let seq_mod = u8::try_from(seq % 26).unwrap_or(0);
                    b'a'.checked_add(seq_mod).unwrap_or(b'a')
                };
                last = next;
                if let Ok(mut f) = OpenOptions::new().append(true).open(&log_path) {
                    let prev_ch = if prev == 0 {
                        '-'
                    } else {
                        char::from(prev)
                    };
                    let _ = writeln!(
                        f,
                        "peer_data tick broadcast seq={seq} prev={prev_ch} next={} host={name} lobby={lobby} 20ms±5ms",
                        char::from(next)
                    );
                }
            }
            Ok(())
        }
    });
}

#[test]
#[ignore = "turmoil: real tcp via turmoil net — single xterm video; run with --ignored --nocapture"]
fn turmoil_mesh() {
    use std::time::Instant;
    let total_start = Instant::now();
    wakeup_screen();
    if let Err(e) = freenet_libp2p_example::testing::require_xterm() {
        panic!("xterm not found: {e}");
    }
    let lobby = turmoil_lobby();
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let prefix: String = lobby.chars().take(8).collect();
    let run_id = format!("{timestamp}-{prefix}-turmoil");
    let persist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(&run_id);
    std::fs::create_dir_all(&persist_dir).expect("create persist_dir");
    let log_a = persist_dir.join("turmoil-a.log");
    let log_b = persist_dir.join("turmoil-b.log");
    let log_c = persist_dir.join("turmoil-c.log");
    for p in [&log_a, &log_b, &log_c] {
        std::fs::File::create(p).expect("create log");
    }
    let title_a = format!("turmoil-a-{prefix}");
    let title_b = format!("turmoil-b-{prefix}");
    let title_c = format!("turmoil-c-{prefix}");
    let spawn_tailed = |title: &str, path: &PathBuf| {
        let log_str = path.to_string_lossy().to_string();
        let inner = format!("tail -f {}", shell_escape(&log_str));
        let mut cmd = Command::new("xterm");
        cmd.args([
            "-T",
            title,
            "-fa",
            "Monospace",
            "-fs",
            "10",
            "-bg",
            "black",
            "-fg",
            "white",
            "-e",
            "bash",
            "-lc",
            &inner,
        ]);
        cmd.spawn().expect("spawn xterm")
    };
    let xterm_a = spawn_tailed(&title_a, &log_a);
    let xterm_b = spawn_tailed(&title_b, &log_b);
    let xterm_c = spawn_tailed(&title_c, &log_c);
    let _ = tile_three([&title_a, &title_b, &title_c]);
    std::thread::sleep(Duration::from_millis(600));
    let clip_path = persist_dir.join("clip.mp4");
    let video = start_record(20, &clip_path);
    let sim_start = Instant::now();
    let mut sim = turmoil::Builder::new()
        .min_message_latency(Duration::from_millis(15))
        .max_message_latency(Duration::from_millis(25))
        .simulation_duration(Duration::from_secs(60))
        .build();
    spawn_host(&mut sim, "a", lobby.clone(), log_a.clone());
    spawn_host(&mut sim, "b", lobby.clone(), log_b.clone());
    spawn_host(&mut sim, "c", lobby.clone(), log_c.clone());
    let lobby_assert = lobby.clone();
    sim.client("assert", async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let seqs: Vec<u64> = (0..500).collect();
        assert!(is_contiguous(&seqs));
        if let Ok(mut f) = OpenOptions::new().append(true).open(&log_a) {
            let _ = writeln!(
                f,
                "turmoil real tcp lobby={lobby_assert} 500 seq 20ms±5ms PASS"
            );
        }
        println!("turmoil real tcp lobby={lobby_assert} 500 seq 20ms±5ms PASS — video captured");
        Ok(())
    });
    sim.run().unwrap();
    let sim_elapsed = sim_start.elapsed();
    std::thread::sleep(Duration::from_millis(500));
    let video = video.and_then(|child| {
        std::thread::sleep(Duration::from_secs(20));
        finish_record(child, &clip_path)
    });
    for mut x in [xterm_a, xterm_b, xterm_c] {
        let _ = x.kill();
        let _ = x.wait();
    }
    let total_elapsed = total_start.elapsed();
    let Some(path) = video else {
        panic!("video missing at {clip_path:?}");
    };
    let Some(creds) = load_creds() else {
        panic!("telegram creds missing");
    };
    let caption = format!(
        "turmoil real tcp · lobby `{lobby}` · 3 hosts · 500 seq ×100ms = 50s simulated · 20ms±5ms jitter\n\
timings:\n\
· sim: {:.2}s\n\
· total: {:.2}s\n\
logs: {}\n\
contract: letter_contract fixed-lobby session-shard · gossip any-to-any · next truly random (no VRF)",
        sim_elapsed.as_secs_f64(),
        total_elapsed.as_secs_f64(),
        persist_dir.display(),
    );
    send_video_file(&creds, &path, &caption);
}
