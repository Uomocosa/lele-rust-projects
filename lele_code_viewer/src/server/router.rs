use std::path::PathBuf;
use std::sync::Arc;

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
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .with_state(state)
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
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
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::Files);
    Html(render::render_file_tree_page(&idx, &cfg)).into_response()
}

// needed helper: source file page
async fn file(
    State(state): State<Arc<server::AppState>>,
    Path((id, path)): Path<(String, String)>,
) -> Response {
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::None);
    let rel = PathBuf::from(&path);
    match render::render_file_page(&idx, &rel, &state.hl, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: item page
async fn item(
    State(state): State<Arc<server::AppState>>,
    Path((id, item_id)): Path<(String, String)>,
) -> Response {
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::None);
    let target = item_id.replace('/', "::");
    match render::render_item_page(&idx, &target, &cfg) {
        Some(html) => Html(html).into_response(),
        None => not_found(),
    }
}

// needed helper: markdown page
async fn md(
    State(state): State<Arc<server::AppState>>,
    Path((id, path)): Path<(String, String)>,
) -> Response {
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::None);
    let rel = PathBuf::from(&path);
    match render::render_md_page(&idx, &rel, &cfg) {
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
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::None);
    let query = params.q.unwrap_or_default();
    Html(render::render_search_page(&idx, &query, &cfg)).into_response()
}

// needed helper: filesystem tree view
async fn file_tree(State(state): State<Arc<server::AppState>>, Path(id): Path<String>) -> Response {
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::Files);
    Html(render::render_file_tree_page(&idx, &cfg)).into_response()
}

// needed helper: dependency tree view
async fn dep_tree(State(state): State<Arc<server::AppState>>, Path(id): Path<String>) -> Response {
    let Some(item) = resolve(&state, &id) else {
        return not_found();
    };
    let Ok(idx) = index_of(&state, &item) else {
        return not_found();
    };
    let cfg = server_cfg(&state, &item, render::ViewKind::Deps);
    Html(render::render_dependency_tree_page(&idx, &cfg)).into_response()
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

// needed helper: cached index for a project
fn index_of(
    state: &server::AppState,
    item: &project::ProjectRef,
) -> Result<Arc<index::SymbolIndex>, Error> {
    project::index_for(&state.registry, item)
}

// needed helper: link config with drawer navigation for a project page
fn server_cfg(
    state: &server::AppState,
    item: &project::ProjectRef,
    view: render::ViewKind,
) -> render::LinkConfig {
    let projects = state
        .projects
        .iter()
        .map(|p| render::NavProject {
            id: p.id.clone(),
            name: p.name.clone(),
        })
        .collect();
    render::LinkConfig {
        prefix: format!("/p/{}/", item.id),
        assets: "/assets/".to_string(),
        html: false,
        nav: Some(render::Nav {
            current: item.id.clone(),
            name: item.name.clone(),
            root: item.root.to_string_lossy().to_string(),
            projects,
            view,
        }),
    }
}

// needed helper: link config for pages without the drawer
fn bare_cfg() -> render::LinkConfig {
    render::LinkConfig {
        prefix: "/".to_string(),
        assets: "/assets/".to_string(),
        html: false,
        nav: None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::router;
    use crate::project;
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
}
