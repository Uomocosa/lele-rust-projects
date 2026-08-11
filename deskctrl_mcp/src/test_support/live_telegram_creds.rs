/// Reads `.env` (mirrors what `main()` loads at runtime) without touching the process-global
/// environment — `std::env::set_var` from a test would leak into every other test running in
/// the same process/binary (e.g. `server::tests::test_usage`, which asserts a fresh `Server`
/// has no Telegram config).
fn dotenv_var(key: &str) -> Option<String> {
    if let Ok(v) = std::env::var(key) {
        return Some(v);
    }
    dotenvy::from_filename_iter(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))
        .ok()?
        .filter_map(Result::ok)
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// Returns real Telegram credentials (from the process env or `.env`), or panics naming exactly
/// which one is missing/blank.
pub fn live_telegram_creds() -> (String, String) {
    let token = non_blank(dotenv_var("TELEGRAM_BOT_TOKEN"));
    let chat_id = non_blank(dotenv_var("TELEGRAM_CHAT_ID"));
    assert!(token.is_some(), "TELEGRAM_BOT_TOKEN required for this test");
    assert!(chat_id.is_some(), "TELEGRAM_CHAT_ID required for this test");
    (token.unwrap(), chat_id.unwrap())
}

// no test_usage necessary
