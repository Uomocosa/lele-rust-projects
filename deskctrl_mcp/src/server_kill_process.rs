use rmcp::model::CallToolResult;

use crate::{Error, Server, server_method};

pub async fn kill_process(
    server: &Server,
    pid: u32,
    send_to_telegram: bool,
) -> Result<CallToolResult, Error> {
    let result = server_method::kill_process(&server.processes, pid).await?;
    if send_to_telegram && let (Some(token), Some(chat_id)) = (&server.bot_token, &server.chat_id) {
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            chat_id.clone(),
            format!("\u{1F6D1} <b>kill_process</b> pid={pid}"),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{Error, Server};

    #[tokio::test]
    async fn test_usage() {
        let server = Server::new();
        let result = super::kill_process(&server, 999, false).await;
        assert!(matches!(result, Err(Error::UnknownPid(999))));
    }
}
