use rmcp::model::CallToolResult;

use crate::{ClickParams, Error, Server, server_method};

pub async fn click_window(server: &Server, params: ClickParams) -> Result<CallToolResult, Error> {
    let send_to_telegram = params.send_to_telegram;
    let note = params.note.clone();
    let (window_id, x, y, button) = (params.window_id.clone(), params.x, params.y, params.button);
    let result = server_method::click_window(params).await?;
    if send_to_telegram && let (Some(token), Some(chat_id)) = (&server.bot_token, &server.chat_id) {
        let message =
            note.unwrap_or_else(|| format!("clicked button {button} at ({x}, {y}) in {window_id}"));
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            chat_id.clone(),
            format!("\u{1F5B1} {}", server_method::html_escape(&message)),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{ClickParams, Error, Server};

    #[tokio::test]
    async fn test_usage() {
        let server = Server::new();
        let params = ClickParams {
            window_id: "nope".to_string(),
            x: 0,
            y: 0,
            button: 1,
            note: None,
            send_to_telegram: false,
        };
        let result = super::click_window(&server, params).await;
        assert!(matches!(result, Err(Error::Window(_))));
    }
}
