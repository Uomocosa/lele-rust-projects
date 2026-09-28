use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;

use crate::Error;
use crate::project;
use crate::server;
use crate::source;

pub async fn serve(
    registry: project::Registry,
    bind: &str,
    options: server::ServeOptions,
) -> Result<(), Error> {
    let settings = project::load_settings(&options.settings_path)?;
    println!(
        "lele_code_viewer: settings {} ({} ignore rules)",
        options.settings_path.display(),
        settings.ignore.len()
    );
    let rules = project::compile_rules(&settings.ignore).unwrap_or_else(|err| {
        println!("lele_code_viewer: {err}; scanning without ignore rules");
        Vec::new()
    });
    let projects = project::discover_projects(&registry.roots, &rules);
    println!("lele_code_viewer: {} projects discovered", projects.len());
    let registry = Arc::new(registry);
    if registry.watch {
        project::start_watcher(&registry)?;
        println!("lele_code_viewer: live reload on (watching opened projects)");
    }
    let state = Arc::new(server::AppState {
        registry,
        projects: RwLock::new(Arc::new(projects)),
        hl: Arc::new(source::highlighter_new()),
        settings: RwLock::new(settings),
        settings_path: options.settings_path,
        self_update: options.self_update,
        scanning: AtomicBool::new(false),
    });
    let listener = tokio::net::TcpListener::bind(bind).await?;
    if let Some(url) = tailscale_url(bind) {
        println!("lele_code_viewer: tailscale url {url}");
    }
    println!("lele_code_viewer: listening on http://{bind}");
    let app = server::router(state);
    axum::serve(listener, app)
        .await
        .map_err(|err| Error::Server(err.to_string()))
}

// needed helper: best-effort tailscale IPv4 url for phone access
fn tailscale_url(bind: &str) -> Option<String> {
    let output = std::process::Command::new("tailscale")
        .args(["ip", "-4"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ip = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if ip.is_empty() {
        return None;
    }
    let port = bind.rsplit(':').next()?;
    Some(format!("http://{ip}:{port}"))
}

// no test_usage necessary
