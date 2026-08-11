use rmcp::model::CallToolResult;

use crate::{Error, Server, SpawnParams, server_method};

pub async fn spawn_process(server: &Server, params: SpawnParams) -> Result<CallToolResult, Error> {
    let send_to_telegram = params.send_to_telegram;
    let desc = format!("{} {}", params.cmd, params.args.join(" "));
    let result = server_method::spawn_process(&server.processes, &server.next_id, params).await?;
    if send_to_telegram && let (Some(token), Some(chat_id)) = (&server.bot_token, &server.chat_id) {
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            chat_id.clone(),
            format!(
                "\u{1F680} <b>spawned</b>: \"{}\"",
                server_method::html_escape(&server_method::truncate(&desc, 200))
            ),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Server, SpawnParams};

    #[tokio::test]
    async fn test_usage() {
        let server = Server::new();
        let params = SpawnParams {
            cmd: "echo".to_string(),
            args: vec!["no-notify".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: false,
        };
        let result = super::spawn_process(&server, params).await;
        assert!(result.is_ok());
        assert!(server.processes.lock().await.contains_key(&1));
        crate::server_method::kill_process(&server.processes, 1)
            .await
            .ok();
    }
}
