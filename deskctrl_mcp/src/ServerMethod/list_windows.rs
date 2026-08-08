use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, WindowInfoMethod};

pub async fn list_windows() -> Result<CallToolResult, Error> {
    let windows = WindowInfoMethod::list()?;

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
    #[tokio::test]
    #[ignore = "requires a live X display"]
    async fn test_usage_live_display() {
        assert!(super::list_windows().await.is_ok());
    }
}
// no test_usage necessary: shells out to wmctrl, so it cannot run without an X display
