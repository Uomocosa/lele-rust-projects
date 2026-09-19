use std::process::Command;

/// # Errors
/// Returns an error if no visible window matches the title.
pub fn window_id(title: &str) -> Result<String, String> {
    let output = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", &format!("{title} \\[")])
        .output()
        .map_err(|err| format!("xdotool search {title}: {err}"))?;
    if !output.status.success() {
        return Err(format!("xdotool cannot find window {title}"));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .next()
        .map(str::to_string)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| format!("xdotool found no id for {title}"))
}

#[cfg(test)]
mod tests {
    use super::window_id;

    #[test]
    fn test_usage() {
        assert!(window_id("no-such-clicker-window-xyz").is_err());
    }
}
