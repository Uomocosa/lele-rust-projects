use crate::Diagnostic;
use crate::ErrorFormat;
use std::io::Write;

pub fn print_diagnostics(diags: &[Diagnostic], error_format: ErrorFormat) {
    let mut stderr = std::io::stderr().lock();
    for d in diags {
        match error_format {
            ErrorFormat::Github => print_github(d, &mut stderr),
            ErrorFormat::Clippy => print_clippy(d, &mut stderr),
        }
    }
}

fn print_clippy(d: &Diagnostic, w: &mut impl Write) {
    let _ = writeln!(
        w,
        "{}:{}:{}: error[{}]: {}",
        d.file.display(),
        d.line,
        d.col,
        d.code,
        d.message
    );
}

fn print_github(d: &Diagnostic, w: &mut impl Write) {
    let _ = writeln!(
        w,
        "::error file={file},line={line},col={col},title={code}::{message}",
        file = d.file.display(),
        line = d.line,
        col = d.col,
        code = d.code,
        message = d.message
    );
}

// no test_usage necessary
