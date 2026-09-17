use super::guard_start;
use super::load_creds;
use super::send_text;

pub struct Guard {
    pub name: String,
    pub start: std::time::Instant,
}

#[rustfmt::skip]
impl Guard {
    #[must_use]
    pub fn start(name: &str) -> Self { guard_start::start(name) }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let status = if std::thread::panicking() {
            "FAILED"
        } else {
            "PASSED"
        };
        let text = format!("{} {} in {:?}", status, self.name, self.start.elapsed());
        let Some(creds) = load_creds::load_creds() else {
            eprintln!("telegram skipped (no creds): {text}");
            return;
        };
        if let Err(err) = send_text::send_text(&creds, &text) {
            eprintln!("telegram send failed: {err}");
        }
    }
}

// no test_usage necessary
