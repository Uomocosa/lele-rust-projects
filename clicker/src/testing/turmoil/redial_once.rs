use std::time::Duration;

use super::constants::PORT;
use super::dial_with_deadline::dial_with_deadline;
use super::write_hello::write_hello;

pub async fn redial_once(
    target: &'static str,
    budget: Duration,
    own_name: String,
    tx: tokio::sync::mpsc::UnboundedSender<(String, turmoil::net::TcpStream)>,
) {
    if let Ok(mut stream) = dial_with_deadline(target, PORT, budget).await
        && write_hello(&mut stream, &own_name).await.is_ok()
        && tx.send((target.to_string(), stream)).is_err()
    {
        tracing::debug!("redial receiver gone for {target}");
    }
}

// no test_usage necessary
