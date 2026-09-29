use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use axum::Form;
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
use axum::routing::post;
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
        .route("/p/{id}/md/{*path}", get(md))
        .route("/p/{id}/raw/{*path}", get(raw))
        .route("/p/{id}/search", get(search))
        .route("/p/{id}/tree/file", get(file_tree))
        .route("/p/{id}/tree/deps", get(dep_tree))
        .route("/p/{id}/events", get(events))
        .route("/settings", get(settings_page).post(save_settings))
        .route("/settings/rescan", post(rescan_now))
        .route("/settings/update", post(self_update))
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .with_state(state)
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[derive(Deserialize)]
struct SettingsForm {
    ignore: String,
    action: Option<String>,
}

#[derive(Deserialize)]
struct NoticeParams {
    notice: Option<String>,
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
    match server::project_list(&state).first() {
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

// needed helper: raw file bytes with a mime type for embeds and downloads
async fn raw(
    State(state): State<Arc<server::AppState>>,
    Path((id, path)): Path<(String, String)>,
) -> Response {
    let Some(page) = load(&state, &id).await else {
        return not_found();
    };
    let rel = PathBuf::from(&path);
    let Some(full) = render::resolve_in_root(&page.idx.root, &rel) else {
        return not_found();
    };
    let bytes = match std::fs::read(&full) {
        Ok(bytes) => bytes,
        Err(_) => return not_found(),
    };
    if bytes.len() > 67_108_864 {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Html("<h1>413</h1><p class=\"muted\">file too large</p>"),
        )
            .into_response();
    }
    ([(CONTENT_TYPE, raw_content_type(&rel))], bytes).into_response()
}

// needed helper: mime type for a raw file by extension
fn raw_content_type(rel: &PathBuf) -> &'static str {
    match rel.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("bmp") => "image/bmp",
        Some("avif") => "image/avif",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogv") => "video/ogg",
        Some("mov") => "video/quicktime",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg" | "oga") => "audio/ogg",
        Some("flac") => "audio/flac",
        Some("m4a") => "audio/mp4",
        Some("json") => "application/json",
        Some("pdf") => "application/pdf",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs" | "cjs") => "text/javascript; charset=utf-8",
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("xml") => "text/xml; charset=utf-8",
        _ => "application/octet-stream",
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

// needed helper: settings page (ignore rules + maintenance actions)
async fn settings_page(
    State(state): State<Arc<server::AppState>>,
    Query(params): Query<NoticeParams>,
) -> Response {
    let notice = match params.notice.as_deref() {
        Some("saved") => Some("Saved. Rescanning with the new rules\u{2026}"),
        Some("rescan") => Some("Rescanning the project list\u{2026}"),
        Some("busy") => Some("A rescan is already running."),
        Some("update") => {
            Some("Update started: the viewer rebuilds and restarts in about a minute.")
        }
        _ => None,
    };
    let rules = settings_of(&state).ignore.join("\n");
    settings_response(
        &state,
        rules,
        notice.map(str::to_string),
        None,
        StatusCode::OK,
    )
}

// needed helper: validate, persist and apply new ignore rules
async fn save_settings(
    State(state): State<Arc<server::AppState>>,
    Form(form): Form<SettingsForm>,
) -> Response {
    let settings = if form.action.as_deref() == Some("defaults") {
        project::default_settings()
    } else {
        project::Settings {
            ignore: form
                .ignore
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
        }
    };
    if let Err(message) = project::compile_rules(&settings.ignore) {
        return settings_response(
            &state,
            form.ignore,
            None,
            Some(message),
            StatusCode::BAD_REQUEST,
        );
    }
    if let Err(err) = project::save_settings(&state.settings_path, &settings) {
        let message = format!("could not save settings: {err}");
        return settings_response(
            &state,
            form.ignore,
            None,
            Some(message),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }
    match state.settings.write() {
        Ok(mut current) => *current = settings,
        Err(poisoned) => *poisoned.into_inner() = settings,
    }
    server::rescan(&state);
    Redirect::to("/settings?notice=saved").into_response()
}

// needed helper: rescan the project list with the current rules
async fn rescan_now(State(state): State<Arc<server::AppState>>) -> Response {
    let notice = if server::rescan(&state) {
        "rescan"
    } else {
        "busy"
    };
    Redirect::to(&format!("/settings?notice={notice}")).into_response()
}

// needed helper: rebuild + restart the viewer service through systemd
async fn self_update(State(state): State<Arc<server::AppState>>) -> Response {
    if !state.self_update {
        return not_found();
    }
    match server::start_self_update() {
        Ok(()) => Redirect::to("/settings?notice=update").into_response(),
        Err(message) => {
            let rules = settings_of(&state).ignore.join("\n");
            settings_response(
                &state,
                rules,
                None,
                Some(message),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

// needed helper: current settings snapshot
fn settings_of(state: &server::AppState) -> project::Settings {
    match state.settings.read() {
        Ok(settings) => settings.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

// needed helper: render the settings page with the given text box content and messages
fn settings_response(
    state: &server::AppState,
    rules: String,
    notice: Option<String>,
    error: Option<String>,
    status: StatusCode,
) -> Response {
    let view = render::SettingsView {
        rules,
        error,
        notice,
        project_count: server::project_list(state).len(),
        scanning: state.scanning.load(Ordering::SeqCst),
        self_update: state.self_update,
        settings_path: project::display_path(&state.settings_path),
    };
    (
        status,
        Html(render::render_settings_page(&bare_cfg(), &view)),
    )
        .into_response()
}

// needed helper: find a project by id
fn resolve(state: &server::AppState, id: &str) -> Option<project::ProjectRef> {
    server::project_list(state)
        .iter()
        .find(|item| item.id == id)
        .cloned()
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
    let projects = server::project_list(state)
        .iter()
        .map(|p| render::NavProject {
            id: p.id.clone(),
            name: p.name.clone(),
            path: project::display_path(&p.root),
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
            projects: std::sync::RwLock::new(Arc::new(Vec::new())),
            hl: Arc::new(source::highlighter_new()),
            settings: std::sync::RwLock::new(project::default_settings()),
            settings_path: std::path::PathBuf::from("settings.toml"),
            self_update: false,
            scanning: std::sync::atomic::AtomicBool::new(false),
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
