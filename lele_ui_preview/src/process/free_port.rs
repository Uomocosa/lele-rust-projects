use std::net::TcpListener;

use crate::Error;

pub fn free_port() -> Result<u16, Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

#[cfg(test)]
mod tests {
    use super::free_port;

    #[test]
    fn test_usage() {
        assert!(free_port().unwrap() > 0);
    }
}
