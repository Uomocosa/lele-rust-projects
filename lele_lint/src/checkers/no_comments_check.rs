use std::path::Path;

use super::no_comments::NoComments;
use crate::common;
use crate::Diagnostic;
use crate::Entry;
use crate::EntryKind;
use crate::Layout;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoComments, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if project.layout != Layout::Methods {
        return diags;
    }
    scan_entries(&project.entries, &project.src_dir, &mut diags);
    if let Some(methods_dir) = &project.methods_dir {
        scan_entries(&project.methods_entries, methods_dir, &mut diags);
    }
    diags
}

// needed helper: scan one entry set for disallowed comments
fn scan_entries(entries: &[Entry], base: &Path, diags: &mut Vec<Diagnostic>) {
    for entry in entries {
        if entry.kind != EntryKind::File {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&entry.absolute_path) else {
            continue;
        };
        for hit in common::find_comments(&content) {
            if !hit.block && is_allowlisted(&hit.text) {
                continue;
            }
            diags.push(Diagnostic {
                file: base.join(&entry.relative_path),
                line: hit.line,
                col: 0,
                code: "E031".to_string(),
                message:
                    "comments are not allowed in the methods layout (code, tests and logs only)"
                        .to_string(),
                severity: Severity::Error,
            });
        }
    }
}

// needed helper: the two sanctioned comment opt-outs
fn is_allowlisted(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "// no test_usage necessary" || trimmed.starts_with("// needed helper:")
}

// no test_usage necessary
