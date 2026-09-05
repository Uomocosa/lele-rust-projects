use crate::Diagnostic;
use crate::Severity;
use std::io::Write;

pub fn print_diagnostics(diags: &[Diagnostic], error_format: &str) {
    let mut stderr = std::io::stderr().lock();
    for d in diags {
        match error_format {
            "github" => print_github(d, &mut stderr),
            _ => print_clippy(d, &mut stderr),
        }
    }
}

fn print_clippy(d: &Diagnostic, w: &mut impl Write) {
    let level = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    let _ = writeln!(
        w,
        "{}:{}:{}: {}[{}]: {}",
        d.file.display(),
        d.line,
        d.col,
        level,
        d.code,
        d.message
    );
}

fn print_github(d: &Diagnostic, w: &mut impl Write) {
    let level = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    let _ = writeln!(
        w,
        "::{level} file={file},line={line},col={col},title={code}::{message}",
        level = level,
        file = d.file.display(),
        line = d.line,
        col = d.col,
        code = d.code,
        message = d.message
    );
}

// no test_usage necessary
