pub fn cleanup_stale() {
    for pattern in ["clicker-xterm-", "/clicker --namespace"] {
        let _ = std::process::Command::new("pkill")
            .args(["-f", pattern])
            .output();
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
}

#[cfg(test)]
mod tests {
    use super::cleanup_stale;

    #[test]
    fn test_usage() {
        let _ = cleanup_stale;
    }
}
