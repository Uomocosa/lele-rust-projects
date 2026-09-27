use std::sync::Arc;

use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use serde::Deserialize;

use crate::assets;
use crate::render;
use crate::server;

pub fn router(state: Arc<server::AppState>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/file/{*path}", get(file))
        .route("/item/{*id}", get(item))
        .route("/md/{*path}", get(md))
        .route("/search", get(search))
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .with_state(state)
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

// needed helper: index page
async fn root(State(state): State<Arc<server::AppState>>) -> Response {
    let cfg = server_cfg();
    Html(render::render_index_page(&state.idx, &cfg)).into_response()
}

// needed helper: source file page
async fn file(State(state): State<Arc<server::AppState>>, Path(path): Path<String>) -> Response {
    let cfg = server_cfg();
    let rel = std::path::PathBuf::from(&path);
    match render::render_file_page(&state.idx, &rel, &state.hl, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: item page
async fn item(State(state): State<Arc<server::AppState>>, Path(id): Path<String>) -> Response {
    let cfg = server_cfg();
    let id = id.replace('/', "::");
    match render::render_item_page(&state.idx, &id, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: markdown page
async fn md(State(state): State<Arc<server::AppState>>, Path(path): Path<String>) -> Response {
    let cfg = server_cfg();
    let rel = std::path::PathBuf::from(&path);
    match render::render_md_page(&state.idx, &rel, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: search page
async fn search(
    State(state): State<Arc<server::AppState>>,
    Query(params): Query<SearchParams>,
) -> Response {
    let cfg = server_cfg();
    let query = params.q.unwrap_or_default();
    Html(render::render_search_page(&state.idx, &query, &cfg)).into_response()
}

// needed helper: bundled stylesheet
async fn style_css() -> Response {
    (
        [(CONTENT_TYPE, "text/css; charset=utf-8")],
        assets::style_css(),
    )
        .into_response()
}

// needed helper: bundled javascript
async fn app_js() -> Response {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        assets::app_js(),
    )
        .into_response()
}

// needed helper: 404 response
fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Html("<h1>404</h1><p class=\"muted\">not found</p>"),
    )
        .into_response()
}

// needed helper: server-side link config (root-relative urls)
fn server_cfg() -> render::LinkConfig {
    render::LinkConfig {
        prefix: "/".to_string(),
        html: false,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::router;
    use crate::index;
    use crate::server;
    use crate::source;

    #[test]
    fn test_usage() {
        let state = Arc::new(server::AppState {
            idx: Arc::new(index::SymbolIndex::default()),
            hl: Arc::new(source::highlighter_new()),
        });
        let _ = router(state);
    }
}
