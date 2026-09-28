use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use crate::Error;
use crate::brp;
use crate::config;
use crate::process;
use crate::report;

pub fn crawl_bevy(
    crate_dir: &Path,
    cfg: &config::BevyConfig,
    out_dir: &Path,
) -> Result<report::Capture, Error> {
    let work = std::env::temp_dir().join("lele-ui-preview").join("bevy");
    if work.exists() {
        std::fs::remove_dir_all(&work)?;
    }
    std::fs::create_dir_all(&work)?;
    let port = process::free_port()?;
    let command = process::substitute(&cfg.command, &[("port", port.to_string())]);
    let mut env = cfg.env.clone();
    env.insert("BRP_EXTRAS_PORT".to_string(), port.to_string());
    let log = work.join("app.log");
    tracing::info!(
        "starting bevy app (first build can take minutes): {}",
        command.join(" ")
    );
    let mut app = process::spawn_app(&command, crate_dir, &env, &log)?;
    let timeout = Duration::from_secs(
        cfg.startup_timeout_secs
            .unwrap_or(brp::DEFAULT_BUILD_AND_START_SECS),
    );
    let result = process::wait_until(&mut app, "bevy app to answer BRP", timeout, || {
        brp::call_brp(port, "rpc.discover", None).is_ok()
    })
    .and_then(|()| drive(cfg, port, &work, out_dir));
    let _ = brp::call_brp(port, "brp_extras/shutdown", None);
    std::thread::sleep(Duration::from_secs(1));
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

// needed helper: generate fixtures from the reflected schema, inject each one and capture it
fn drive(
    cfg: &config::BevyConfig,
    port: u16,
    work: &Path,
    out_dir: &Path,
) -> Result<report::Capture, Error> {
    let resource = resolve_resource(port, &cfg.resource)?;
    let schema = brp::call_brp(port, "registry.schema", None)?;
    let max = cfg.max_states.unwrap_or(brp::DEFAULT_FIXTURE_CAP);
    let fixtures = brp::fixtures(&schema, &resource, max);
    if fixtures.is_empty() {
        return Err(Error::Brp(format!(
            "no fixtures could be generated from the schema of {resource}"
        )));
    }
    let settle = Duration::from_millis(cfg.settle_ms.unwrap_or(brp::DEFAULT_FIXTURE_SETTLE_MS));
    let screen = resource
        .rsplit("::")
        .next()
        .unwrap_or(&resource)
        .to_string();
    let mut capture = report::Capture::default();
    capture.notes.push(format!(
        "{} fixtures generated from the reflected schema of {resource} (cap {max})",
        fixtures.len()
    ));
    let mut first_id: Option<String> = None;
    for (position, fixture) in fixtures.iter().enumerate() {
        brp::call_brp(
            port,
            "world.insert_resources",
            Some(json!({ "resource": resource, "value": fixture.value })),
        )?;
        let bytes = stable_capture(port, work, position, settle)?;
        let pixel_hash: String = blake3::hash(&bytes).to_hex().chars().take(16).collect();
        let id = format!("{}-s{position:03}", brp::GROUP);
        if let Some(twin) = capture.states.iter().find(|s| s.pixel_hash == pixel_hash) {
            capture.notes.push(format!(
                "fixture '{}' renders identically to {}; not stored",
                fixture.label, twin.id
            ));
            continue;
        }
        let png = report::store_png(out_dir, brp::GROUP, &screen, &id, &fixture.label, &bytes)?;
        let fingerprint: String = blake3::hash(fixture.value.to_string().as_bytes())
            .to_hex()
            .chars()
            .take(16)
            .collect();
        capture.states.push(report::StateRecord {
            id: id.clone(),
            group: brp::GROUP.to_string(),
            screen: screen.clone(),
            path: vec![format!("set {screen}: {}", fixture.label)],
            location: fixture.value.to_string(),
            png,
            fingerprint: format!("{screen}|data={fingerprint}"),
            pixel_hash,
        });
        match &first_id {
            None => first_id = Some(id),
            Some(base) => capture.edges.push(report::EdgeRecord {
                from: base.clone(),
                to: id,
                action: format!("set {screen}: {}", fixture.label),
            }),
        }
    }
    Ok(capture)
}

// needed helper: capture until two consecutive screenshots match, i.e. the UI finished reacting
fn stable_capture(
    port: u16,
    work: &Path,
    position: usize,
    settle: Duration,
) -> Result<Vec<u8>, Error> {
    let mut previous: Option<Vec<u8>> = None;
    for attempt in 0..brp::STABLE_CAPTURE_ATTEMPTS {
        std::thread::sleep(settle);
        let shot = work.join(format!("shot-{position}-{attempt}.png"));
        let bytes = capture(port, &shot)?;
        if previous.as_ref() == Some(&bytes) {
            return Ok(bytes);
        }
        previous = Some(bytes);
    }
    previous.ok_or_else(|| Error::Brp("no screenshot captured".to_string()))
}

// needed helper: one screenshot, waiting out a capture that is still being finalized
fn capture(port: u16, shot: &Path) -> Result<Vec<u8>, Error> {
    let params = json!({ "path": shot.display().to_string() });
    let mut last_error = None;
    for _ in 0..brp::BUSY_RETRIES {
        match brp::call_brp(port, "brp_extras/screenshot", Some(params.clone())) {
            Ok(_) => return Ok(std::fs::read(shot)?),
            Err(Error::Brp(message)) if message.contains("already in progress") => {
                last_error = Some(Error::Brp(message));
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(other) => return Err(other),
        }
    }
    Err(last_error.unwrap_or_else(|| Error::Brp("screenshot never started".to_string())))
}

// needed helper: map a short resource name (e.g. "Multiplayer") to its full type path
fn resolve_resource(port: u16, wanted: &str) -> Result<String, Error> {
    let listed = brp::call_brp(port, "world.list_resources", None)?;
    let suffix = format!("::{wanted}");
    listed
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|path| *path == wanted || path.ends_with(&suffix))
        .map(str::to_string)
        .ok_or_else(|| {
            Error::Brp(format!(
                "resource {wanted} is not registered for reflection"
            ))
        })
}

// no test_usage necessary
