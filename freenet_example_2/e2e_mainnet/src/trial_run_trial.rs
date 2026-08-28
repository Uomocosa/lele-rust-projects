use std::path::Path;
use std::time::{Duration, Instant};

use crate::Error;
use crate::config;
use crate::evaluate;
use crate::find_window_by_pid;
use crate::finish_record;
use crate::instance;
use crate::kill_all_instances;
use crate::launch_instances;
use crate::list_windows;
use crate::new_run_dir;
use crate::outcome;
use crate::read_trace;
use crate::reconcile_analyze;
use crate::reconcile_result;
use crate::start_record;
use crate::tile_windows;
use crate::trial_result;
use crate::wait_all_ready;
use crate::wakeup_screen;
use crate::window_info;

const POLL_SECS: u64 = 5;

struct TeardownGuard;

#[rustfmt::skip]
impl Drop for TeardownGuard {
    fn drop(&mut self) { let _ = kill_all_instances::kill_all_instances(); }
}

pub fn run_trial(
    cfg: &config::Config,
    bin: &Path,
    mode: &str,
    rep: usize,
) -> Result<trial_result::TrialResult, Error> {
    let run_dir = new_run_dir::new_run_dir(&format!("{mode}-r{rep}"))?;
    println!("[trial {mode} r{rep}] run-dir: {}", run_dir.root.display());

    wakeup_screen::wakeup_screen();
    let instances = launch_instances::launch_instances(bin, &run_dir, cfg.instances, mode)?;
    let _guard = TeardownGuard;

    let mut ready = true;
    if let Err(e) = wait_all_ready::wait_all_ready(&instances, cfg.timeout_secs) {
        ready = false;
        println!("[trial {mode} r{rep}] not all connected: {e}");
    }

    let elapsed = wait_for_convergence(cfg, &instances, mode, rep);

    let windows = find_windows(&instances);
    tile_windows::tile_windows(&windows)?;

    wakeup_screen::wakeup_screen();
    let video = if cfg.no_video {
        None
    } else {
        let secs = cfg.clip_secs;
        let path = run_dir.root.join("clip.mp4");
        match start_record::start_record(secs, &path) {
            Ok(child) => {
                std::thread::sleep(Duration::from_secs(secs));
                finish_record::finish_record(child, &path).ok()
            }
            Err(_) => None,
        }
    };

    let mut traces = Vec::new();
    for inst in &instances {
        traces.push(read_trace::read_trace(&inst.log_path).unwrap_or_default());
    }
    let rr: reconcile_result::ReconcileResult = reconcile_analyze::analyze(&traces, mode);

    let outcome: outcome::Outcome = evaluate::evaluate(&instances, elapsed)?;
    let error_sigs = outcome.error_sigs;
    let put_count = outcome.put_count;
    let _killed = kill_all_instances::kill_all_instances().is_ok();

    Ok(trial_result::TrialResult {
        mode: mode.to_string(),
        rep,
        ready,
        put_count,
        error_sigs,
        final_counts: rr.final_counts,
        reconciled: rr.reconciled,
        latency_secs: rr.latency_secs,
        expected_union: rr.expected_union,
        merged_correct: rr.merged_correct,
        aggregated: rr.aggregated,
        video,
        run_label: run_dir.label.clone(),
    })
}

// needed helper:
fn wait_for_convergence(
    cfg: &config::Config,
    instances: &[instance::Instance],
    mode: &str,
    rep: usize,
) -> u64 {
    let start = Instant::now();
    let mut stable = 0u64;
    loop {
        let traces: Vec<_> = instances
            .iter()
            .map(|i| read_trace::read_trace(&i.log_path).unwrap_or_default())
            .collect();
        let rr: reconcile_result::ReconcileResult = reconcile_analyze::analyze(&traces, mode);
        if rr.merged_correct == Some(true) {
            stable += POLL_SECS;
            if stable >= cfg.settle_secs {
                let elapsed = start.elapsed().as_secs();
                println!("[trial {mode} r{rep}] converged after {elapsed}s");
                return elapsed;
            }
        } else {
            stable = 0;
        }
        let elapsed = start.elapsed().as_secs();
        if elapsed >= cfg.timeout_secs {
            println!("[trial {mode} r{rep}] timed out waiting for convergence ({elapsed}s)");
            return elapsed;
        }
        std::thread::sleep(Duration::from_secs(POLL_SECS));
    }
}

// needed helper:
fn find_windows(instances: &[instance::Instance]) -> Vec<(usize, window_info::WindowInfo)> {
    let all = list_windows::list_windows().unwrap_or_default();
    instances
        .iter()
        .filter_map(|inst| {
            find_window_by_pid::find_window_by_pid(&all, inst.pid).map(|w| (inst.index, w))
        })
        .collect()
}

// no test_usage necessary — exercised via the live e2e run
