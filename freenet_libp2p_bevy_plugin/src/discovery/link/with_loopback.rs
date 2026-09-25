#[must_use]
pub fn with_loopback(addrs: &[String]) -> Vec<String> {
    let mut out: Vec<String> = addrs.to_vec();
    for addr in addrs {
        if let Some(rest) = addr.strip_prefix("/ip4/")
            && let Some(port) = rest.split("/tcp/").nth(1)
        {
            let ip = rest.split('/').next().unwrap_or_default();
            if ip != "127.0.0.1" && !ip.is_empty() && !port.is_empty() {
                out.push(format!("/ip4/127.0.0.1/tcp/{port}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::with_loopback;

    #[test]
    fn test_usage() {
        assert_eq!(with_loopback(&[]), Vec::<String>::new());
        let addrs = with_loopback(&["/ip4/1.2.3.4/tcp/9000".to_string()]);
        assert_eq!(addrs.len(), 2);
        assert!(addrs.contains(&"/ip4/127.0.0.1/tcp/9000".to_string()));
    }
}
