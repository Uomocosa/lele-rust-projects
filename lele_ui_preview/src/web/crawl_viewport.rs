use std::collections::{HashMap, VecDeque};
use std::path::Path;

use crate::Error;
use crate::cdp;
use crate::config;
use crate::report;
use crate::web;

pub fn crawl_viewport(
    session: &mut cdp::Session,
    target: &web::Target,
    viewport: &config::Viewport,
    start: &str,
    out_dir: &Path,
) -> Result<report::Capture, Error> {
    cdp::set_viewport(session, viewport)?;
    let group = viewport.name.as_str();
    let mut capture = report::Capture::default();
    let mut states: Vec<web::WebState> = Vec::new();
    let mut by_fingerprint: HashMap<String, String> = HashMap::new();
    let mut by_location: HashMap<String, String> = HashMap::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    let root = web::WebState {
        location: start.to_string(),
        steps: Vec::new(),
        path: Vec::new(),
        depth: 0,
    };
    web::reach_state(session, target, &root)?;
    let first = web::probe(session)?;
    let root_id = record(session, target, group, out_dir, &root, &first, &mut capture)?;
    by_fingerprint.insert(fingerprint_of(target, &first), root_id.clone());
    by_location.insert(root.location.clone(), root_id);
    states.push(root);
    queue.push_back(0);
    let mut budget_hit = false;
    while let Some(position) = queue.pop_front() {
        let Some(state) = states.get(position).cloned() else {
            continue;
        };
        let Some(from) = capture.states.get(position).map(|s| s.id.clone()) else {
            continue;
        };
        if state.depth >= target.max_depth {
            continue;
        }
        web::reach_state(session, target, &state)?;
        let here = web::probe(session)?;
        for action in web::next_actions(&here, target) {
            let label = web::action_label(&action);
            let mut candidate = successor(&state, &action, &label);
            if let Some(known) = by_location
                .get(&candidate.location)
                .filter(|_| candidate.steps.is_empty())
            {
                push_edge(&mut capture, &from, known, &label);
                continue;
            }
            if capture.states.len() >= target.max_states {
                budget_hit = true;
                break;
            }
            let seen = match visit(session, target, &mut candidate) {
                Ok(seen) => seen,
                Err(error) => {
                    capture
                        .notes
                        .push(format!("{group}: {from} -> {label} failed: {error}"));
                    continue;
                }
            };
            let fingerprint = fingerprint_of(target, &seen);
            if let Some(known) = by_fingerprint.get(&fingerprint) {
                if candidate.steps.is_empty() {
                    by_location.insert(candidate.location.clone(), known.clone());
                }
                push_edge(&mut capture, &from, known, &label);
                continue;
            }
            let id = record(
                session,
                target,
                group,
                out_dir,
                &candidate,
                &seen,
                &mut capture,
            )?;
            push_edge(&mut capture, &from, &id, &label);
            by_fingerprint.insert(fingerprint, id.clone());
            if candidate.steps.is_empty() {
                by_location.insert(candidate.location.clone(), id);
            }
            queue.push_back(states.len());
            states.push(candidate);
        }
        if budget_hit {
            break;
        }
    }
    if budget_hit {
        capture.notes.push(format!(
            "{group}: stopped at max_states = {}; raise it in [ui_preview.web] to see more",
            target.max_states
        ));
    }
    Ok(capture)
}

// needed helper: the state reached by applying one action to another state
fn successor(state: &web::WebState, action: &web::Action, label: &str) -> web::WebState {
    let mut path = state.path.clone();
    path.push(label.to_string());
    let depth = state.depth.saturating_add(1);
    if let web::Action::Navigate { location, .. } = action {
        return web::WebState {
            location: location.clone(),
            steps: Vec::new(),
            path,
            depth,
        };
    }
    let mut steps = state.steps.clone();
    steps.push(action.clone());
    web::WebState {
        location: state.location.clone(),
        steps,
        path,
        depth,
    }
}

// needed helper: load a candidate state; a click that changed page becomes a plain location
fn visit(
    session: &mut cdp::Session,
    target: &web::Target,
    candidate: &mut web::WebState,
) -> Result<web::Probe, Error> {
    web::reach_state(session, target, candidate)?;
    let seen = web::probe(session)?;
    let start_path = candidate
        .location
        .split(['?', '#'])
        .next()
        .unwrap_or_default();
    if !candidate.steps.is_empty()
        && seen.url != start_path
        && seen.url.split('?').next() != Some(start_path)
    {
        candidate.location = format!("{}{}", seen.url, seen.hash);
        candidate.steps.clear();
    }
    Ok(seen)
}

// needed helper: route-level screen name plus structural fingerprint
fn fingerprint_of(target: &web::Target, seen: &web::Probe) -> String {
    web::fingerprint(&screen_of(target, seen), seen, &target.patterns)
}

// needed helper: the route pattern a page belongs to
fn screen_of(target: &web::Target, seen: &web::Probe) -> String {
    let path = seen.url.split('?').next().unwrap_or(&seen.url);
    web::match_route(&target.patterns, path).unwrap_or_else(|| path.to_string())
}

// needed helper: append a transition
fn push_edge(capture: &mut report::Capture, from: &str, to: &str, action: &str) {
    capture.edges.push(report::EdgeRecord {
        from: from.to_string(),
        to: to.to_string(),
        action: action.to_string(),
    });
}

// needed helper: screenshot the current page and add it as a new state
fn record(
    session: &mut cdp::Session,
    target: &web::Target,
    group: &str,
    out_dir: &Path,
    state: &web::WebState,
    seen: &web::Probe,
    capture: &mut report::Capture,
) -> Result<String, Error> {
    let id = format!("{}-s{:03}", report::slug(group), capture.states.len());
    let screen = screen_of(target, seen);
    let bytes = cdp::screenshot(session)?;
    let label = state.path.last().map_or("start", String::as_str);
    let png = report::store_png(out_dir, group, &screen, &id, label, &bytes)?;
    let pixel_hash: String = blake3::hash(&bytes).to_hex().chars().take(16).collect();
    capture.states.push(report::StateRecord {
        id: id.clone(),
        group: group.to_string(),
        screen,
        path: state.path.clone(),
        location: format!("{}{}", seen.url, seen.hash),
        png,
        fingerprint: fingerprint_of(target, seen),
        pixel_hash,
    });
    Ok(id)
}

// no test_usage necessary
