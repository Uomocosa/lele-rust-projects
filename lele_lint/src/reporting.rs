// lele_lint: allow E001
// no test_usage necessary
use std::io::Write;

use crate::checker::{Diagnostic, Severity};

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
        d.message,
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
        message = d.message,
    );
}

pub fn print_checker_list(checkers: &[Box<dyn crate::checker::Checker>]) {
    for c in checkers {
        println!("{:>5}  {}", c.code(), c.name());
    }
}
