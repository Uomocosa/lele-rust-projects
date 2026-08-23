use super::creds::Creds;

pub fn load_creds() -> Option<Creds> {
    dotenvy::dotenv().ok();
    let token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").ok();
    match (token, chat_id) {
        (Some(token), Some(chat_id)) if !token.is_empty() && !chat_id.is_empty() => {
            Some(Creds { token, chat_id })
        }
        _ => None,
    }
}

// no test_usage necessary
