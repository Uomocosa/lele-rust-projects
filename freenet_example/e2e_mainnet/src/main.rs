use std::process::exit;

use e2e_mainnet::build_game;
use e2e_mainnet::build_report;
use e2e_mainnet::error::Error;
use e2e_mainnet::evaluate;
use e2e_mainnet::find_window_by_pid;
use e2e_mainnet::finish_record;
use e2e_mainnet::instance;
use e2e_mainnet::kill_all_instances;
use e2e_mainnet::list_windows;
use e2e_mainnet::load_creds;
use e2e_mainnet::new_run_dir;
use e2e_mainnet::parse_config;
use e2e_mainnet::send_text;
use e2e_mainnet::send_video;
use e2e_mainnet::spawn_one;
use e2e_mainnet::start_record;
use e2e_mainnet::tile_windows;
use e2e_mainnet::wait_all_ready;
use e2e_mainnet::window_info::WindowInfo;

const RECORD_MARGIN_SECS: u64 = 10;

struct TeardownGuard;

#[rustfmt::skip]
impl Drop for TeardownGuard {
    fn drop(&mut self) { let _ = kill_all_instances::kill_all_instances(); }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("e2e-mainnet: {e}");
        exit(1);
    }
}

// needed helper:
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

    let instances = spawn_instances(&bin, &run_dir, cfg.instances)?;
    let _teardown_guard = TeardownGuard;

    let mut failure: Option<String> = None;
    if let Err(e) = wait_all_ready::wait_all_ready(&instances, cfg.timeout_secs) {
        failure = Some(e.to_string());
        eprintln!("{e}");
    }

    let found = list_windows::list_windows().unwrap_or_default();
    let windows = filter_windows(&found, &instances);
    println!("matched {} windows by pid", windows.len());
    tile_windows::tile_windows(&windows)?;

    let recorder = if cfg.no_video {
        None
    } else {
        let secs = cfg.settle_secs + RECORD_MARGIN_SECS;
        let path = run_dir.root.join("session.mp4");
        Some((start_record::start_record(secs, &path)?, path, secs))
    };

    std::thread::sleep(std::time::Duration::from_secs(cfg.settle_secs));

    let outcome = evaluate::evaluate(&instances, cfg.settle_secs)?;
    let killed = kill_all_instances::kill_all_instances().is_ok();
    println!("killed: {killed}");

    let mut report_text = build_report::build_report(
        &run_dir.contract_params,
        &run_dir.root.display().to_string(),
        &instances,
        &outcome,
        killed,
    );
    if let Some(f) = &failure {
        report_text = format!("{report_text}\nfailure: {f}");
    }
    println!("{report_text}");

    let all_ready = outcome.instances.iter().all(|o| o.ready);
    if let Some(creds) = creds {
        if let Some((child, path, secs)) = recorder {
            let bytes = finish_record::finish_record(child, &path)?;
            let caption = format!(
                "freenet_example e2e · contract={} · {} instances · {secs}s",
                run_dir.contract_params,
                instances.len()
            );
            let step = send_video::send_video(&creds, &bytes, Some(&caption))?;
            println!("{step}");
        }
        let step = send_text::send_text(&creds, &report_text)?;
        println!("{step}");
    }

    if all_ready && outcome.error_sigs.is_empty() {
        Ok(())
    } else {
        Err(Error::Assertion(format!(
            "run did not pass: ready={all_ready} errs={} (report+videos sent either way)",
            outcome.error_sigs.len()
        )))
    }
}

/// Spawn all clicker instances (xterm windows running the standalone mainnet-client app).
// needed helper:
fn spawn_instances(
    bin: &std::path::Path,
    run_dir: &e2e_mainnet::run_dir::RunDir,
    count: usize,
) -> Result<Vec<instance::Instance>, Error> {
    let mut instances = Vec::with_capacity(count);
    for index in 0..count {
        instances.push(spawn_one::spawn_one(index, bin, run_dir)?);
    }
    Ok(instances)
}

// needed helper:
fn filter_windows(
    all: &[WindowInfo],
    instances: &[instance::Instance],
) -> Vec<(usize, WindowInfo)> {
    instances
        .iter()
        .filter_map(|inst| {
            find_window_by_pid::find_window_by_pid(all, inst.pid).map(|w| (inst.index, w))
        })
        .collect()
}
