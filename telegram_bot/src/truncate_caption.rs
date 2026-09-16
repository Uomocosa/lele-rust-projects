const MAX_CAPTION_CHARS: usize = 1000;

#[must_use]
pub fn truncate_caption(caption: &str) -> String {
    if caption.chars().count() <= MAX_CAPTION_CHARS {
        return caption.to_string();
    }
    let kept: String = caption
        .chars()
        .take(MAX_CAPTION_CHARS.saturating_sub(3))
        .collect();
    format!("{kept}...")
}

#[cfg(test)]
mod tests {
    use super::truncate_caption;

    #[test]
    fn test_usage() {
        assert_eq!(truncate_caption("short"), "short");
        let long = "x".repeat(2000);
        let cut = truncate_caption(&long);
        assert_eq!(cut.chars().count(), 1000);
        assert!(cut.ends_with("..."));
        let emoji = "✅".repeat(2000);
        let cut_emoji = truncate_caption(&emoji);
        assert_eq!(cut_emoji.chars().count(), 1000);
    }
}
