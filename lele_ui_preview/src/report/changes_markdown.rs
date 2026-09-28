use std::fmt::Write;

use crate::report;

pub fn changes_markdown(previous: Option<&report::Manifest>, current: &report::Manifest) -> String {
    let mut out = format!("# UI preview changes: {}\n\n", current.crate_name);
    let Some(before) = previous else {
        out.push_str("No previous run found: this run is the baseline.\n");
        let _ = writeln!(out, "\n{} states captured.", current.states.len());
        return out;
    };
    let status_of = |state: &report::StateRecord| report::state_status(Some(before), state);
    let changed: Vec<&report::StateRecord> = current
        .states
        .iter()
        .filter(|s| status_of(s) == report::STATUS_CHANGED)
        .collect();
    let added: Vec<&report::StateRecord> = current
        .states
        .iter()
        .filter(|s| status_of(s) == report::STATUS_NEW)
        .collect();
    let removed: Vec<&report::StateRecord> = before
        .states
        .iter()
        .filter(|old| {
            !current
                .states
                .iter()
                .any(|s| s.group == old.group && s.fingerprint == old.fingerprint)
        })
        .collect();
    let same = current
        .states
        .len()
        .saturating_sub(changed.len())
        .saturating_sub(added.len());
    let _ = writeln!(
        out,
        "Compared with the previous run: {} changed, {} new, {} removed, {same} unchanged.\n",
        changed.len(),
        added.len(),
        removed.len()
    );
    if changed.is_empty() && added.is_empty() && removed.is_empty() {
        out.push_str("No UI state changed.\n");
        return out;
    }
    section(
        &mut out,
        "Changed (same state, different pixels)",
        &changed,
        true,
    );
    section(&mut out, "New states", &added, true);
    section(
        &mut out,
        "Removed states (no longer reachable)",
        &removed,
        false,
    );
    out
}

// needed helper: one markdown list of states
fn section(out: &mut String, title: &str, states: &[&report::StateRecord], link: bool) {
    if states.is_empty() {
        return;
    }
    let _ = writeln!(out, "## {title}\n");
    for state in states {
        let path = if state.path.is_empty() {
            "start".to_string()
        } else {
            state.path.join(" > ")
        };
        let target = if link {
            format!(" ([png]({}))", state.png)
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "- `{}` {} `{}`: {path}{target}",
            state.id, state.group, state.screen
        );
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::changes_markdown;
    use crate::report;

    fn record(id: &str, fingerprint: &str, pixel_hash: &str) -> report::StateRecord {
        report::StateRecord {
            id: id.to_string(),
            group: "mobile".to_string(),
            screen: "/".to_string(),
            path: vec!["open menu".to_string()],
            location: "/".to_string(),
            png: format!("mobile/root/{id}.png"),
            fingerprint: fingerprint.to_string(),
            pixel_hash: pixel_hash.to_string(),
        }
    }

    #[test]
    fn test_usage() {
        let before = report::Manifest {
            crate_name: "demo".to_string(),
            states: vec![record("s000", "a", "1"), record("s001", "b", "1")],
            ..report::Manifest::default()
        };
        let after = report::Manifest {
            crate_name: "demo".to_string(),
            states: vec![record("s000", "a", "2"), record("s001", "c", "1")],
            ..report::Manifest::default()
        };
        let text = changes_markdown(Some(&before), &after);
        assert!(text.contains("1 changed, 1 new, 1 removed, 0 unchanged"));
        assert!(changes_markdown(Some(&before), &before).contains("No UI state changed."));
        assert!(changes_markdown(None, &after).contains("baseline"));
    }
}
