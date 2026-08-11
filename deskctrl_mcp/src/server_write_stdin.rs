use rmcp::model::CallToolResult;

use crate::{Error, Server, WriteStdinParams, server_method};

pub async fn write_stdin(
    server: &Server,
    params: WriteStdinParams,
) -> Result<CallToolResult, Error> {
    let send_to_telegram = params.send_to_telegram;
    let (pid, text) = (params.pid, params.text.clone());
    let result = server_method::write_stdin(&server.processes, params).await?;
    if send_to_telegram && let (Some(token), Some(chat_id)) = (&server.bot_token, &server.chat_id) {
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            chat_id.clone(),
            format!(
                "\u{2328}\u{FE0F} <b>write_stdin</b> pid={pid}: \"{}\"",
                server_method::html_escape(&server_method::truncate(&text, 120))
            ),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{Error, Server, WriteStdinParams};

    #[tokio::test]
    async fn test_usage() {
        let server = Server::new();
        let params = WriteStdinParams {
            pid: 999,
            text: "hello".to_string(),
            send_to_telegram: false,
        };
        let result = super::write_stdin(&server, params).await;
        assert!(matches!(result, Err(Error::UnknownPid(999))));
    }
}
