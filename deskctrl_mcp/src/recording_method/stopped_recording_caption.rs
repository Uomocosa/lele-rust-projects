use super::stopped_recording::StoppedRecording;

pub fn caption(stopped: &StoppedRecording) -> String {
    let mb = stopped.size_bytes as f64 / (1024.0 * 1024.0);
    format!(
        "\u{1F3AC} Session video \u{2014} {} \u{2014} {}s, {:.1} MB",
        stopped.target, stopped.duration_secs, mb
    )
}

#[cfg(test)]
mod tests {
    use crate::recording_method::StoppedRecording;

    #[test]
    fn test_usage() {
        let stopped = StoppedRecording {
            path: "p.mp4".to_string(),
            duration_secs: 42,
            size_bytes: 5 * 1024 * 1024,
            target: "window 0x1 (demo)".to_string(),
        };
        let text = stopped.caption();
        assert!(text.contains("42s"));
        assert!(text.contains("5.0 MB"));
        assert!(text.contains("demo"));
    }
}
