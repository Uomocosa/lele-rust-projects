use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, window_info_method};

pub async fn list_windows() -> Result<CallToolResult, Error> {
    let windows = window_info_method::list()?;

    if windows.is_empty() {
        return Ok(CallToolResult::success(vec![ContentBlock::text(
            "no open windows".to_string(),
        )]));
    }

    let mut table = String::from("window_id   pid    geometry           title\n");
    for w in &windows {
        table.push_str(&format!(
            "{:<11} {:<6} {:<18} {}\n",
            w.id,
            w.pid,
            w.geometry(),
            w.title
        ));
    }
    table.push_str("\npass a window_id (or pid/title) to screenshot to capture just that window");

    Ok(CallToolResult::success(vec![ContentBlock::text(table)]))
}

#[cfg(test)]
mod tests {
    use crate::{test_support, window_info_method};

    /// Spawns a real xterm so the desktop always has a non-`Desktop` window to list, then
    /// verifies the wmctrl parse path surfaces it.
    #[tokio::test]
    async fn test_usage_live_display() {
        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().lock().await;

        let mut child = std::process::Command::new("xterm")
            .spawn()
            .expect("spawning xterm for live list_windows test");
        std::thread::sleep(std::time::Duration::from_millis(800));

        let windows = window_info_method::list().expect("list_windows for live list_windows test");
        let window = windows
            .iter()
            .find(|w| w.pid == child.id())
            .expect("spawned xterm must appear in list_windows");

        let result = super::list_windows().await.unwrap();

        let _ = child.kill();
        let _ = child.wait();

        let text = format!("{result:?}");
        assert!(text.contains("window_id") && text.contains("pid"));
        assert!(text.contains(&window.id));
    }
}
// no test_usage necessary: shells out to wmctrl, so it cannot run without an X display
