use crate::testing;

#[must_use]
pub fn load_creds() -> Option<testing::Creds> {
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))
        .or_else(|_| dotenvy::dotenv());
    let token = std::env::var("TELEGRAM_BOT_TOKEN")
        .or_else(|_| std::env::var("TG_BOT_TOKEN"))
        .ok()?;
    let chat_id = std::env::var("TELEGRAM_CHAT_ID")
        .or_else(|_| std::env::var("TG_CHAT_ID"))
        .ok()?;
    if token.is_empty() || chat_id.is_empty() {
        return None;
    }
    Some(testing::Creds { token, chat_id })
}

#[cfg(test)]
mod tests {
    use super::load_creds;

    #[test]
    fn test_usage() {
        let _ = load_creds();
    }
}
