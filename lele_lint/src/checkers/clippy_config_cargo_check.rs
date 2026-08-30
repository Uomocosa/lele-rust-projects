use std::path::PathBuf;

use super::clippy_config_cargo::ClippyConfigCargo;
use crate::diagnostic;
use crate::project;
use crate::severity;

const REQUIRED_LINTS: &[&str] = &[
    "unwrap_used",
    "expect_used",
    "indexing_slicing",
    "arithmetic_side_effects",
    "unreachable",
    "unimplemented",
    "unchecked_time_subtraction",
    "todo",
    "string_slice",
    "panic_in_result_fn",
    "panic",
    "exit",
    "as_conversions",
];

pub(crate) fn check(
    _self: &ClippyConfigCargo,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
    let cargo_path = project.root.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(_) => {
            return vec![diagnostic::Diagnostic {
                file: cargo_path,
                line: 1,
                col: 0,
                code: ClippyConfigCargo::CODE.to_string(),
                message: "Cargo.toml not found or unreadable — add [lints.clippy] with minimum clippy config".to_string(),
                severity: severity::Severity::Error,
            }];
        }
    };
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            return vec![diagnostic::Diagnostic {
                file: cargo_path,
                line: 1,
                col: 0,
                code: ClippyConfigCargo::CODE.to_string(),
                message: format!("Cargo.toml parse error: {e}"),
                severity: severity::Severity::Error,
            }];
        }
    };
    let clippy_table = resolve_clippy_table(&value, project, &cargo_path);
    match clippy_table {
        Some(table) => {
            let mut diags = Vec::new();
            for key in ["pedantic", "nursery"] {
                match table.get(key) {
                    Some(v) if is_deny_with_priority(v, true) => {}
                    _ => diags.push(diag_for_key(&cargo_path, key, "must be { level = \"deny\", priority = -1 }")),
                }
            }
            for key in REQUIRED_LINTS {
                match table.get(*key) {
                    Some(v) if is_deny_with_priority(v, false) => {}
                    _ => diags.push(diag_for_key(&cargo_path, key, "must be \"deny\"")),
                }
            }
            diags
        }
        None => vec![diagnostic::Diagnostic {
            file: cargo_path.clone(),
            line: 1,
            col: 0,
            code: ClippyConfigCargo::CODE.to_string(),
            message: "missing [lints.clippy] in Cargo.toml — add minimum clippy config (pedantic/nursery + 13 deny lints)".to_string(),
            severity: severity::Severity::Error,
        }],
    }
}

fn resolve_clippy_table(
    value: &toml::Value,
    project: &project::Project,
    cargo_path: &PathBuf,
) -> Option<toml::value::Table> {
    if let Some(table) = value
        .get("lints")
        .and_then(|v| v.get("clippy"))
        .and_then(|v| v.as_table())
    {
        return Some(table.clone());
    }
    if let Some(workspace_table) = value
        .get("workspace")
        .and_then(|v| v.get("lints"))
        .and_then(|v| v.get("clippy"))
        .and_then(|v| v.as_table())
    {
        return Some(workspace_table.clone());
    }
    if inherits_workspace_lints(value) {
        if let Some(parent) = find_workspace_root(project) {
            let ws_cargo = parent.join("Cargo.toml");
            if ws_cargo != *cargo_path {
                if let Ok(ws_content) = std::fs::read_to_string(&ws_cargo) {
                    if let Ok(ws_value) = ws_content.parse::<toml::Value>() {
                        if let Some(ws_table) = ws_value
                            .get("workspace")
                            .and_then(|v| v.get("lints"))
                            .and_then(|v| v.get("clippy"))
                            .and_then(|v| v.as_table())
                        {
                            return Some(ws_table.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

// needed helper:
fn inherits_workspace_lints(value: &toml::Value) -> bool {
    value
        .get("lints")
        .and_then(|v| v.get("workspace"))
        .and_then(|v| v.as_bool())
        == Some(true)
}

// needed helper:
fn find_workspace_root(project: &project::Project) -> Option<PathBuf> {
    let mut cur = project.root.parent().map(|p| p.to_path_buf());
    while let Some(dir) = cur {
        if dir.join("Cargo.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(dir.join("Cargo.toml")) {
                if let Ok(val) = content.parse::<toml::Value>() {
                    if val.get("workspace").is_some() {
                        return Some(dir);
                    }
                }
            }
        }
        let parent = dir.parent().map(|p| p.to_path_buf());
        if parent.is_none() || parent == Some(dir.clone()) {
            break;
        }
        cur = parent;
    }
    None
}

// needed helper:
fn is_deny_with_priority(value: &toml::Value, require_priority: bool) -> bool {
    match value {
        toml::Value::String(s) => s == "deny",
        toml::Value::Table(t) => {
            let level_ok = t.get("level").and_then(|v| v.as_str()) == Some("deny");
            if !level_ok {
                return false;
            }
            if require_priority {
                match t.get("priority").and_then(|v| v.as_integer()) {
                    Some(-1) => true,
                    _ => false,
                }
            } else {
                true
            }
        }
        _ => false,
    }
}

// needed helper:
fn diag_for_key(path: &PathBuf, key: &str, expected: &str) -> diagnostic::Diagnostic {
    diagnostic::Diagnostic {
        file: path.clone(),
        line: 1,
        col: 0,
        code: ClippyConfigCargo::CODE.to_string(),
        message: format!(
            "Cargo.toml [lints.clippy].{key} {expected} — minimum clippy config requires it"
        ),
        severity: severity::Severity::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::is_deny_with_priority;

    #[test]
    fn test_usage() {
        let deny_str = toml::Value::String("deny".to_string());
        assert!(is_deny_with_priority(&deny_str, false));
        let table: toml::Value =
            r#"value = { level = "deny", priority = -1 }"#.parse::<toml::Value>().unwrap()["value"]
                .clone();
        assert!(is_deny_with_priority(&table, true));
        let bad: toml::Value =
            r#"value = { level = "warn", priority = -1 }"#.parse::<toml::Value>().unwrap()["value"]
                .clone();
        assert!(!is_deny_with_priority(&bad, true));
    }
}
