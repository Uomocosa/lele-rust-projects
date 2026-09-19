use std::collections::HashMap;

use bevy::prelude::App;
use freenet_libp2p_bevy_plugin::p2p;
use tokio::sync::mpsc::UnboundedSender;

use super::conn_writer::ConnWriter;
use super::envelope::Envelope;
use super::read_loop::read_loop;
use crate::clicker;

pub fn register(
    app: &mut App,
    outbound: &mut HashMap<String, ConnWriter>,
    inbox_tx: &UnboundedSender<Envelope>,
    peer: &str,
    stream: turmoil::net::TcpStream,
) {
    let (reader, writer) = tokio::io::split(stream);
    outbound.insert(peer.to_string(), writer);
    tokio::spawn(read_loop(reader, peer.to_string(), inbox_tx.clone()));
    app.world_mut()
        .resource_mut::<p2p::Events<clicker::CursorMsg>>()
        .push(p2p::Event::PeerConnected(peer.to_string()));
}

// no test_usage necessary
