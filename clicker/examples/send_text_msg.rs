use telegram_bot;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("send_text_msg: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let text: Vec<String> = std::env::args().skip(1).collect();
    if text.is_empty() {
        return Err("usage: send_text_msg <message...>".to_string());
    }
    let Some(creds) = telegram_bot::load_creds() else {
        return Err("missing TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID in clicker/.env".to_string());
    };
    let id = telegram_bot::send_text(&creds, &text.join(" ")).map_err(|err| err.to_string())?;
    println!("sent text as message {id}");
    Ok(())
}
