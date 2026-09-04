use futures::StreamExt;
use libp2p::Swarm;
use libp2p::request_response::{self, Message};
use libp2p::swarm::SwarmEvent;

use crate::relay;

pub async fn drive_swarm(swarm: &mut Swarm<relay::Behaviour>, gossip: &mut relay::GossipState) {
    if let SwarmEvent::Behaviour(request_response::Event::Message { peer, message, .. }) =
        swarm.select_next_some().await
        && let Message::Request {
            request: relay::LetterRequest(frame),
            channel,
            ..
        } = message
    {
        let accept = gossip.should_accept(&frame);
        if accept {
            gossip.insert(frame.clone());
            println!(
                "peer_data recv seq={} prev={} next={} from={peer} via Freenet-discovered dial",
                frame.seq,
                char::from(frame.prev),
                char::from(frame.next)
            );
            tracing::debug!(seq=frame.seq, %peer, "recv via Freenet-discovered addr");
        }
        let _ = swarm
            .behaviour_mut()
            .send_response(channel, relay::LetterResponse(accept));
    }
}

// no test_usage necessary
