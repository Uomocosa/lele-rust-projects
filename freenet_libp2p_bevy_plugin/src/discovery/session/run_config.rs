use crate::p2p;

use super::super::config::Config;
use super::super::freenet_endpoint::FreenetEndpoint;
use super::super::link::NetLink;

pub struct RunConfig {
    pub config: Config,
    pub endpoint: FreenetEndpoint,
    pub link: NetLink,
    pub tap: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub ready: tokio::sync::watch::Receiver<Option<p2p::Ready>>,
    pub commands: tokio::sync::mpsc::UnboundedReceiver<super::super::Command>,
    pub multiplayer: tokio::sync::mpsc::UnboundedSender<super::super::Multiplayer>,
    pub events: tokio::sync::mpsc::UnboundedSender<super::super::Event>,
}
// no test_usage necessary
