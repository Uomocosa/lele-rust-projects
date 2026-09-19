use tokio::sync::mpsc::UnboundedSender;

use super::read_hello::read_hello;

pub async fn accept_loop(
    listener: turmoil::net::TcpListener,
    accept_tx: UnboundedSender<(String, turmoil::net::TcpStream)>,
) {
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            break;
        };
        if let Ok(origin) = read_hello(&mut stream).await
            && accept_tx.send((origin, stream)).is_err()
        {
            break;
        }
    }
}

// no test_usage necessary
