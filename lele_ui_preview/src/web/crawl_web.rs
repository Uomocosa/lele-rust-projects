use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::Error;
use crate::cdp;
use crate::config;
use crate::process;
use crate::report;
use crate::web;

pub fn crawl_web(
    crate_dir: &Path,
    cfg: &config::WebConfig,
    out_dir: &Path,
) -> Result<report::Capture, Error> {
    let work = std::env::temp_dir().join("lele-ui-preview").join("web");
    if work.exists() {
        std::fs::remove_dir_all(&work)?;
    }
    std::fs::create_dir_all(&work)?;
    let fixture = match &cfg.fixture {
        Some(relative) => {
            let prepared = work.join("fixture");
            web::prepare_fixture(&crate_dir.join(relative), &prepared)?;
            prepared
        }
        None => crate_dir.to_path_buf(),
    };
    let port = process::free_port()?;
    let command = process::substitute(
        &cfg.command,
        &[
            ("port", port.to_string()),
            ("fixture", fixture.display().to_string()),
        ],
    );
    let log = work.join("app.log");
    tracing::info!("starting web app: {}", command.join(" "));
    let mut app = process::spawn_app(&command, crate_dir, &cfg.env, &log)?;
    let timeout = Duration::from_secs(
        cfg.startup_timeout_secs
            .unwrap_or(web::DEFAULT_STARTUP_TIMEOUT_SECS),
    );
    let result = process::wait_until(&mut app, "web app to accept connections", timeout, || {
        TcpStream::connect(("127.0.0.1", port)).is_ok()
    })
    .and_then(|()| with_browser(crate_dir, cfg, &work, port, out_dir));
    process::stop_app(&mut app);
    match result {
        Ok(capture) => {
            let _ = std::fs::remove_dir_all(&work);
            Ok(capture)
        }
        Err(error) => Err(Error::Config(format!(
            "{error} (app log kept at {})",
            log.display()
        ))),
    }
}

// needed helper: drive headless Chrome over every configured viewport
fn with_browser(
    crate_dir: &Path,
    cfg: &config::WebConfig,
    work: &Path,
    port: u16,
    out_dir: &Path,
) -> Result<report::Capture, Error> {
    let target = web::Target {
        base_url: format!("http://127.0.0.1:{port}"),
        patterns: web::route_patterns(crate_dir, &cfg.routes_from),
        keys: web::scan_keys(crate_dir, &cfg.keys_from),
        skip_paths: cfg.skip_paths.clone(),
        fill_text: cfg
            .fill_text
            .clone()
            .unwrap_or_else(|| web::DEFAULT_FILL_TEXT.to_string()),
        settle_ms: cfg.settle_ms.unwrap_or(web::DEFAULT_SETTLE_MS),
        max_states: cfg.max_states.unwrap_or(web::DEFAULT_MAX_STATES),
        max_depth: cfg.max_depth.unwrap_or(web::DEFAULT_MAX_DEPTH),
    };
    let mut capture = report::Capture::default();
    capture.notes.push(format!(
        "screens from code: {} route patterns in {:?}",
        target.patterns.len(),
        cfg.routes_from
    ));
    capture.notes.push(format!(
        "keys from code: {:?} in {:?}",
        target.keys, cfg.keys_from
    ));
    let chrome = cfg.chrome.as_deref().unwrap_or(web::DEFAULT_CHROME);
    let profile: PathBuf = work.join("chrome-profile");
    let mut browser = cdp::launch_chrome(chrome, &profile)?;
    let start = cfg.start.as_deref().unwrap_or(web::DEFAULT_START);
    let result = cdp::open_page(&browser).and_then(|mut session| {
        for viewport in &cfg.viewports {
            tracing::info!(
                "crawling viewport {} ({}x{})",
                viewport.name,
                viewport.width,
                viewport.height
            );
            let viewport_capture =
                web::crawl_viewport(&mut session, &target, viewport, start, out_dir)?;
            tracing::info!(
                "viewport {}: {} states",
                viewport.name,
                viewport_capture.states.len()
            );
            capture.states.extend(viewport_capture.states);
            capture.edges.extend(viewport_capture.edges);
            capture.notes.extend(viewport_capture.notes);
        }
        Ok(())
    });
    let _ = browser.child.kill();
    let _ = browser.child.wait();
    result.map(|()| capture)
}

// no test_usage necessary
