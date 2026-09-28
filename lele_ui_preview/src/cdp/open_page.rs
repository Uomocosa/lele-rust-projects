use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;
use tungstenite::stream::MaybeTlsStream;

use crate::Error;
use crate::cdp;

pub fn open_page(browser: &cdp::Browser) -> Result<cdp::Session, Error> {
    let url = format!("http://127.0.0.1:{}/json/new?about:blank", browser.port);
    let body = ureq::put(&url)
        .send_empty()
        .map_err(|e| Error::Http(format!("{url}: {e}")))?
        .body_mut()
        .read_to_string()
        .map_err(|e| Error::Http(format!("{url}: {e}")))?;
    let target: Value = serde_json::from_str(&body)?;
    let ws_url = target
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Cdp(format!("no webSocketDebuggerUrl in {body}")))?;
    let (socket, _) =
        tungstenite::connect(ws_url).map_err(|e| Error::Cdp(format!("{ws_url}: {e}")))?;
    if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
        set_timeout(stream)?;
    }
    Ok(cdp::Session { socket, next_id: 0 })
}

// needed helper: never block forever on a hung browser
fn set_timeout(stream: &TcpStream) -> Result<(), Error> {
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    Ok(())
}

// no test_usage necessary
