use std::sync::Arc;

use crate::Error;
use crate::project;
use crate::server;
use crate::source;

pub async fn serve(registry: project::Registry, bind: &str) -> Result<(), Error> {
    let projects = project::discover_projects(&registry.roots);
    println!("lele_code_viewer: {} projects discovered", projects.len());
    let state = Arc::new(server::AppState {
        registry: Arc::new(registry),
        projects: Arc::new(projects),
        hl: Arc::new(source::highlighter_new()),
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
