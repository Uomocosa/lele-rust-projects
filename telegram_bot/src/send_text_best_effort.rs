use super::load_creds;
use super::send_text;

pub fn send_text_best_effort(text: &str) {
    let Some(creds) = load_creds::load_creds() else {
        eprintln!("telegram skipped (no creds): {text}");
        return;
    };
    if let Err(err) = send_text::send_text(&creds, text) {
        eprintln!("telegram send failed: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::send_text_best_effort;

    #[test]
    fn test_usage() {
        send_text_best_effort("");
    }
}
