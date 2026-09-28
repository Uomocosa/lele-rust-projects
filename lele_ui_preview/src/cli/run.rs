use clap::Parser;

use crate::Error;
use crate::brp;
use crate::cli;
use crate::config;
use crate::report;
use crate::web;

pub fn run() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();
    let parsed = cli::Args::parse();
    let crate_dir = parsed.crate_dir.canonicalize()?;
    let cfg = config::load_config(&crate_dir)?;
    let out_dir = crate_dir.join(config::OUTPUT_DIR);
    let previous = report::load_previous(&out_dir);
    report::reset_dir(&out_dir)?;
    let mut manifest = report::Manifest {
        crate_name: crate_dir
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().to_string()),
        ..report::Manifest::default()
    };
    let mut drivers = Vec::new();
    if let Some(web_cfg) = &cfg.web {
        drivers.push("web");
        merge(
            &mut manifest,
            web::crawl_web(&crate_dir, web_cfg, &out_dir)?,
        );
    }
    if let Some(bevy_cfg) = &cfg.bevy {
        drivers.push("bevy");
        merge(
            &mut manifest,
            brp::crawl_bevy(&crate_dir, bevy_cfg, &out_dir)?,
        );
    }
    manifest.driver = drivers.join("+");
    report::write_report(&out_dir, &manifest, previous.as_ref())?;
    println!(
        "lele-ui-preview: {} states, {} transitions -> {}",
        manifest.states.len(),
        manifest.edges.len(),
        out_dir.join(report::INDEX_FILE).display()
    );
    Ok(())
}

// needed helper: append one driver's capture to the manifest
fn merge(manifest: &mut report::Manifest, capture: report::Capture) {
    manifest.states.extend(capture.states);
    manifest.edges.extend(capture.edges);
    manifest.notes.extend(capture.notes);
}

// no test_usage necessary
