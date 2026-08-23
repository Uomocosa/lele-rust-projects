use std::net::ToSocketAddrs;
use std::time::Duration;

use tokio::net::TcpStream;

/// Preflight for network-dependent e2e tests: the production node-startup path
/// (`roster::start_embedded_node`) joins the real public Freenet mainnet gateway list, so a
/// test built on it can fail simply because the sandbox/CI runner has no internet access. This
/// distinguishes that case from the bug the test exists to catch, instead of surfacing both as
/// the same ambiguous 60s timeout.
pub async fn check_internet_access() -> Result<(), String> {
    let addrs = "1.1.1.1:443"
        .to_socket_addrs()
        .map_err(|e| format!("no internet access — cannot run this network-dependent e2e test: DNS/socket resolution failed: {e}"))?;
    let addr = addrs.into_iter().next().ok_or_else(|| {
        "no internet access — cannot run this network-dependent e2e test: empty address list"
            .to_string()
    })?;

    tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr))
        .await
        .map_err(|_| {
            "no internet access — cannot run this network-dependent e2e test: connect timed out"
                .to_string()
        })?
        .map_err(|e| {
            format!("no internet access — cannot run this network-dependent e2e test: {e}")
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_internet_access;

    #[tokio::test]
    async fn test_usage() {
        assert!(check_internet_access().await.is_ok());
    }
}
