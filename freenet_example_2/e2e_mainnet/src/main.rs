use std::process::exit;

use e2e_mainnet_2::Error;
use e2e_mainnet_2::build_game;
use e2e_mainnet_2::build_report;
use e2e_mainnet_2::load_creds;
use e2e_mainnet_2::parse_config;
use e2e_mainnet_2::send_text;
use e2e_mainnet_2::send_video;
use e2e_mainnet_2::trial_result;
use e2e_mainnet_2::trial_run_trial;

fn main() {
    if let Err(e) = run() {
        eprintln!("e2e-mainnet-2: {e}");
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

    let mut trials: Vec<trial_result::TrialResult> = Vec::new();
    for mode in cfg.modes() {
        for rep in 0..cfg.repeats {
            let t = trial_run_trial::run_trial(&cfg, &bin, mode, rep)?;
            if let (Some(creds), Some(bytes)) = (&creds, &t.video) {
                let caption = format!(
                    "freenet_example_2 reconcile · {}-r{} · reconciled={}",
                    t.mode, t.rep, t.reconciled
                );
                let _ = send_video::send_video(creds, bytes, Some(&caption));
            }
            trials.push(t);
        }
    }

    let report_text = build_report::build_report(&trials, cfg.instances);
    println!("{report_text}");
    if let Some(creds) = creds {
        let _ = send_text::send_text(&creds, &report_text);
    }

    let ok = trials.iter().all(|t| t.ready && t.error_sigs.is_empty());
    println!("result: {}", if ok { "PASS" } else { "FAIL" });
    if ok {
        Ok(())
    } else {
        Err(Error::Assertion(
            "some trial had instances not all connected or raised error signatures".to_string(),
        ))
    }
}
