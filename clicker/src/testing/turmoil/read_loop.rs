use freenet_libp2p_bevy_plugin::p2p;
use tokio::io::AsyncRead;

use super::envelope::Envelope;
use super::read_event::read_event;

pub async fn read_loop<R>(
    mut stream: R,
    origin: String,
    inbox: tokio::sync::mpsc::UnboundedSender<Envelope>,
) where
    R: AsyncRead + Unpin,
{
    loop {
        if let Ok(env) = read_event(&mut stream).await {
            if inbox.send(env).is_err() {
                break;
            }
        } else {
            let event = p2p::Event::PeerDisconnected(origin.clone());
            inbox.send((origin.clone(), 0, event)).ok();
            break;
        }
    }
}

// no test_usage necessary
