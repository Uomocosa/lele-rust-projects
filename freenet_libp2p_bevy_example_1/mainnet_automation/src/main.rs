use std::process::exit;

use mainnet_automation::Error;
use mainnet_automation::build_game;
use mainnet_automation::build_report;
use mainnet_automation::evaluate;
use mainnet_automation::find_window_by_pid;
use mainnet_automation::finish_record;
use mainnet_automation::kill_all_instances;
use mainnet_automation::launch_instances;
use mainnet_automation::list_windows;
use mainnet_automation::load_creds;
use mainnet_automation::move_instance;
use mainnet_automation::new_run_dir;
use mainnet_automation::parse_config;
use mainnet_automation::send_text;
use mainnet_automation::send_video;
use mainnet_automation::start_record;
use mainnet_automation::tile_windows;
use mainnet_automation::wait_all_converged;
use mainnet_automation::window_info::WindowInfo;

const MOVE_HOLD_MS: u64 = 1500;
const MOVE_PAUSE_MS: u64 = 400;
const SETTLE_SECS: u64 = 6;
const RECORD_MARGIN_SECS: u64 = 3;

struct TeardownGuard;

#[rustfmt::skip]
impl Drop for TeardownGuard {
    fn drop(&mut self) { let _ = kill_all_instances::kill_all_instances(); }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("mainnet-automation: {e}");
        exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cfg = parse_config::parse_config();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let creds = if cfg.no_telegram {
        None
    } else {
        load_creds::load_creds()
    };

    let bin = build_game::build_game(&cfg)?;
    println!("binary: {}", bin.display());

    let run_dir = new_run_dir::new_run_dir()?;
    println!("run-dir: {}", run_dir.root.display());
    println!("contract: {}", run_dir.contract_params);

    let instances = launch_instances::launch_instances(&bin, &run_dir, cfg.instances)?;
    println!("launched {} instances", instances.len());
    let _teardown_guard = TeardownGuard;

    wait_all_converged::wait_all_converged(&instances, cfg.timeout_secs)?;
    println!("all {} instances mutually converged", instances.len());

    let found = find_windows(&instances)?;
    println!("matched {} windows by pid", found.len());
    tile_windows::tile_windows(&found)?;
    verify_tiling(&found)?;

    let recorder = if cfg.no_video {
        None
    } else {
        let secs = record_secs(instances.len());
        let path = run_dir.root.join("session.mp4");
        Some((start_record::start_record(secs, &path)?, path, secs))
    };

    for (_, win) in &found {
        move_instance::move_instance(&win.id, true, MOVE_HOLD_MS)?;
        std::thread::sleep(std::time::Duration::from_millis(MOVE_PAUSE_MS));
        move_instance::move_instance(&win.id, false, MOVE_HOLD_MS)?;
        std::thread::sleep(std::time::Duration::from_millis(MOVE_PAUSE_MS));
    }

    std::thread::sleep(std::time::Duration::from_secs(SETTLE_SECS));

    let outcome = evaluate::evaluate(&instances)?;
    let killed = kill_all_instances::kill_all_instances().is_ok();
    println!("killed: {killed}");

    let report_text = build_report::build_report(
        &run_dir.contract_params,
        &run_dir.root.display().to_string(),
        &instances,
        &outcome,
        killed,
    );
    println!("{report_text}");

    if let Some(creds) = creds {
        if let Some((child, path, secs)) = recorder {
            let bytes = finish_record::finish_record(child, &path)?;
            let caption = format!(
                "local-mainnet run · contract={} · {} instances · {secs}s",
                run_dir.contract_params,
                instances.len()
            );
            let step = send_video::send_video(&creds, &bytes, Some(&caption))?;
            println!("{step}");
        }
        let step = send_text::send_text(&creds, &report_text)?;
        println!("{step}");
    }

    if outcome.all_moved()
        && outcome.all_converged()
        && outcome.error_sigs.is_empty()
        && outcome.within_flicker_budget(cfg.allowed_flicker_secs)
    {
        Ok(())
    } else {
        Err(Error::Assertion(format!(
            "run did not pass: moved={} converged={} errs={} flap={:.1}s/{}s",
            found.len(),
            outcome.all_converged(),
            outcome.error_sigs.len(),
            outcome.max_offline_secs,
            cfg.allowed_flicker_secs
        )))
    }
}

// needed helper:
fn find_windows(
    instances: &[mainnet_automation::instance::Instance],
) -> Result<Vec<(usize, WindowInfo)>, Error> {
    let all = list_windows::list_windows()?;
    Ok(instances
        .iter()
        .filter_map(|inst| {
            find_window_by_pid::find_window_by_pid(&all, inst.pid).map(|w| (inst.index, w))
        })
        .collect())
}

// needed helper:
fn verify_tiling(found: &[(usize, WindowInfo)]) -> Result<(), Error> {
    let after = list_windows::list_windows()?;
    for (idx, win) in found {
        let now = find_window_by_pid::find_window_by_pid(&after, win.pid);
        match now {
            Some(w) => println!(
                "tiled instance-{idx}: {}x{}+{},{}",
                w.width, w.height, w.x, w.y
            ),
            None => println!("tiling: instance-{idx} window no longer visible"),
        }
    }
    Ok(())
}

// needed helper:
fn record_secs(instances: usize) -> u64 {
    let drive_ms = instances as u64 * 2 * (MOVE_HOLD_MS + MOVE_PAUSE_MS);
    let total_ms = drive_ms + SETTLE_SECS * 1000;
    total_ms.div_ceil(1000) + RECORD_MARGIN_SECS
}
