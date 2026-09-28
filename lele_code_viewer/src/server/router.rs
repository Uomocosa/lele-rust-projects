use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::response::sse::KeepAlive;
use axum::response::sse::Sse;
use axum::routing::get;
use serde::Deserialize;

use crate::Error;
use crate::assets;
use crate::index;
use crate::project;
use crate::render;
use crate::server;

pub fn router(state: Arc<server::AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/p/{id}", get(project_index))
        .route("/p/{id}/", get(project_index))
        .route("/p/{id}/file/{*path}", get(file))
        .route("/p/{id}/item/{*item_id}", get(item))
        .route("/p/{id}/md/{*path}", get(md))
        .route("/p/{id}/search", get(search))
        .route("/p/{id}/tree/file", get(file_tree))
        .route("/p/{id}/tree/deps", get(dep_tree))
        .route("/p/{id}/events", get(events))
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .with_state(state)
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[derive(Deserialize)]
struct EventParams {
    since: Option<u64>,
}

struct Loaded {
    item: project::ProjectRef,
    idx: Arc<index::SymbolIndex>,
    version: u64,
}

// needed helper: redirect to the first discovered project
async fn home(State(state): State<Arc<server::AppState>>) -> Response {
    match state.projects.first() {
        Some(first) => Redirect::to(&format!("/p/{}/", first.id)).into_response(),
        None => Html(render::page_shell(
            &bare_cfg(),
            "Lele Code Viewer",
            "<h1>Lele Code Viewer</h1><p class=\"muted\">no Rust projects found</p>",
        ))
        .into_response(),
    }
}

// needed helper: project landing page (the file tree view)
async fn project_index(
    State(state): State<Arc<server::AppState>>,
    Path(id): Path<String>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::Files, "*");
    Html(render::render_file_tree_page(&page.idx, &cfg)).into_response()
}

// needed helper: source file page
async fn file(
    State(state): State<Arc<server::AppState>>,
    Path((id, path)): Path<(String, String)>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::None, &path);
    let rel = PathBuf::from(&path);
    match render::render_file_page(&page.idx, &rel, &state.hl, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: item page
async fn item(
    State(state): State<Arc<server::AppState>>,
    Path((id, item_id)): Path<(String, String)>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let target = item_id.replace('/', "::");
    let watch = page
        .idx
        .by_id
        .get(&target)
        .and_then(|&i| page.idx.items.get(i))
        .map(|found| found.file.to_string_lossy().to_string())
        .unwrap_or_default();
    let cfg = server_cfg(&state, &page, render::ViewKind::None, &watch);
    match render::render_item_page(&page.idx, &target, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: markdown page
async fn md(
    State(state): State<Arc<server::AppState>>,
    Path((id, path)): Path<(String, String)>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::None, &path);
    let rel = PathBuf::from(&path);
    match render::render_md_page(&page.idx, &rel, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: search page
async fn search(
    State(state): State<Arc<server::AppState>>,
    Path(id): Path<String>,
    Query(params): Query<SearchParams>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::None, "");
    let query = params.q.unwrap_or_default();
    Html(render::render_search_page(&page.idx, &query, &cfg)).into_response()
}

// needed helper: filesystem tree view
async fn file_tree(State(state): State<Arc<server::AppState>>, Path(id): Path<String>) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::Files, "*");
    Html(render::render_file_tree_page(&page.idx, &cfg)).into_response()
}

// needed helper: dependency tree view
async fn dep_tree(State(state): State<Arc<server::AppState>>, Path(id): Path<String>) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let cfg = server_cfg(&state, &page, render::ViewKind::Deps, "*.rs");
    Html(render::render_dependency_tree_page(&page.idx, &cfg)).into_response()
}

// needed helper: server-sent change events for one project (live mode only)
async fn events(
    State(state): State<Arc<server::AppState>>,
    Path(id): Path<String>,
    Query(params): Query<EventParams>,
) -> Response {
    if !state.registry.watch {
        return not_found();
    }
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let registry = state.registry.clone();
    let watched = item.clone();
    if tokio::task::spawn_blocking(move || project::ensure_watch(&registry, &watched))
        .await
        .is_err()
    {
        return not_found();
    }
    let rx = state.registry.live.lock().ok().and_then(|live| {
        live.projects
            .get(&item.id)
            .map(|entry| entry.notify.subscribe())
    });
    let Some(rx) = rx else {
        return not_found();
    };
    let stream = server::events_stream(
        state.registry.clone(),
        item.id,
        params.since.unwrap_or(0),
        rx,
    );
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(45)))
        .into_response()
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

// needed helper: find a project by id
fn resolve(state: &server::AppState, id: &str) -> Option<project::ProjectRef> {
    state.projects.iter().find(|item| item.id == id).cloned()
}

// needed helper: resolve a project, start watching it (live mode) and get its index off the runtime
async fn load(state: &Arc<server::AppState>, id: &str) -> Option<Loaded> {
    let item = resolve(state, id)?;
    let registry = state.registry.clone();
    let target = item.clone();
    let built = tokio::task::spawn_blocking(move || {
        let version = if registry.watch {
            project::ensure_watch(&registry, &target)
        } else {
            0
        };
        project::index_for(&registry, &target).map(|idx| (idx, version))
    })
    .await
    .map_err(|err| Error::Server(err.to_string()))
    .and_then(|result| result);
    let (idx, version) = built.ok()?;
    Some(Loaded { item, idx, version })
}

// needed helper: link config with drawer navigation (and live state) for a project page
fn server_cfg(
    state: &server::AppState,
    page: &Loaded,
    view: render::ViewKind,
    watch: &str,
) -> render::LinkConfig {
    let item = &page.item;
    let projects = state
        .projects
        .iter()
        .map(|p| render::NavProject {
            id: p.id.clone(),
            name: p.name.clone(),
        })
        .collect();
    let live = state.registry.watch.then(|| render::Live {
        events: format!("/p/{}/events", item.id),
        version: page.version,
        watch: watch.to_string(),
        recent: project::recent_changes(&state.registry, &item.id),
    });
    render::LinkConfig {
        prefix: format!("/p/{}/", item.id),
        assets: "/".to_string(),
        html: false,
        nav: Some(render::Nav {
            current: item.id.clone(),
            name: item.name.clone(),
            root: item.root.to_string_lossy().to_string(),
            projects,
            view,
            live,
        }),
    }
}

// needed helper: link config for pages without the drawer
fn bare_cfg() -> render::LinkConfig {
    render::LinkConfig {
        prefix: "/".to_string(),
        assets: "/".to_string(),
        html: false,
        nav: None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{bare_cfg, router};
    use crate::project;
    use crate::render;
    use crate::server;
    use crate::source;

    #[test]
    fn test_usage() {
        let state = Arc::new(server::AppState {
            registry: Arc::new(project::Registry::default()),
            projects: Arc::new(Vec::new()),
            hl: Arc::new(source::highlighter_new()),
        });
        let _ = router(state);
    }

    #[test]
    fn test_asset_links_resolve_to_served_route() {
        assert_eq!(
            render::href(&bare_cfg(), render::LinkKind::Asset, "style.css"),
            "/assets/style.css"
        );
    }
}
