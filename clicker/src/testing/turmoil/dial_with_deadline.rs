use std::time::Duration;

pub async fn dial_with_deadline(
    target: &'static str,
    port: u16,
    budget: Duration,
) -> Result<turmoil::net::TcpStream, &'static str> {
    let start = tokio::time::Instant::now();
    loop {
        let remaining = budget.saturating_sub(start.elapsed());
        if let Ok(Ok(stream)) =
            tokio::time::timeout(remaining, turmoil::net::TcpStream::connect((target, port))).await
        {
            return Ok(stream);
        }
        if start.elapsed() >= budget {
            return Err("timeout");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// no test_usage necessary
