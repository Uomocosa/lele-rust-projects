use std::sync::Arc;

use crate::Error;
use crate::index;
use crate::server;
use crate::source;

pub async fn serve(idx: index::SymbolIndex, bind: &str) -> Result<(), Error> {
    let state = Arc::new(server::AppState {
        idx: Arc::new(idx),
        hl: Arc::new(source::highlighter_new()),
    });
    let listener = tokio::net::TcpListener::bind(bind).await?;
    if let Some(url) = tailscale_url(bind) {
        println!("lele_docs: tailscale url {url}");
    }
    println!("lele_docs: listening on http://{bind}");
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
