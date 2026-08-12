use rmcp::model::CallToolResult;

use crate::{Error, SendKeysParams, Server, server_method};

pub async fn send_keys(server: &Server, params: SendKeysParams) -> Result<CallToolResult, Error> {
    let send_to_telegram = params.send_to_telegram;
    let note = params.note.clone();
    let window_id = params.window_id.clone();
    let inputs = params.inputs.clone();
    let result = server_method::send_keys(params).await?;
    if send_to_telegram && let (Some(token), Some(chat_id)) = (&server.bot_token, &server.chat_id) {
        let message = note.unwrap_or_else(|| {
            format!(
                "{} in {window_id}",
                server_method::summarize_inputs(&inputs)
            )
        });
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            chat_id.clone(),
            format!("\u{2328} {}", server_method::html_escape(&message)),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{Error, KeyboardInput, KeyboardKey, SendKeysParams, Server};

    #[tokio::test]
    async fn test_usage() {
        let server = Server::new();
        let params = SendKeysParams {
            window_id: "nope".to_string(),
            inputs: vec![KeyboardInput::Chord {
                keys: vec![
                    KeyboardKey("ctrl".to_string()),
                    KeyboardKey("a".to_string()),
                ],
            }],
            note: None,
            send_to_telegram: false,
        };
        let result = super::send_keys(&server, params).await;
        assert!(matches!(result, Err(Error::Window(_))));
    }
}
