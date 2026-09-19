use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::App;
use futures_util::future::LocalBoxFuture;

use super::conn_writer::ConnWriter;
use super::done::Done;
use super::envelope::Envelope;
use super::up_link_converge_to;
use super::up_link_handshake;
use super::up_link_park_until_done;
use super::up_link_pump;
use super::up_link_settle;
use crate::testing;

pub struct UpLink {
    pub app: App,
    pub outbound: HashMap<String, ConnWriter>,
    pub inbox_tx: tokio::sync::mpsc::UnboundedSender<Envelope>,
    pub inbox_rx: tokio::sync::mpsc::UnboundedReceiver<Envelope>,
    pub accept_rx: tokio::sync::mpsc::UnboundedReceiver<(String, turmoil::net::TcpStream)>,
    pub accept_tx: tokio::sync::mpsc::UnboundedSender<(String, turmoil::net::TcpStream)>,
    pub seen: HashSet<(String, u64)>,
    pub seq: u64,
    pub dead: HashSet<String>,
    pub last_sync: HashMap<String, Duration>,
    pub link_down_after: Duration,
    pub dial_failed: Vec<String>,
    pub redial_targets: Vec<&'static str>,
    pub redial_every: Duration,
    pub last_redial: HashMap<String, Duration>,
}

#[rustfmt::skip]
impl UpLink {
    pub fn pump<'a>(&'a mut self, name: &'a str) -> LocalBoxFuture<'a, ()> { up_link_pump::pump(self, name) }
    pub fn handshake<'a>(&'a mut self, name: &'a str) -> LocalBoxFuture<'a, ()> { up_link_handshake::handshake(self, name) }
    pub fn converge_to<'a>(&'a mut self, name: &'a str, want: testing::MeshCount) -> LocalBoxFuture<'a, ()> { up_link_converge_to::converge_to(self, name, want) }
    pub fn settle<'a>(&'a mut self, name: &'a str, iters: u32) -> LocalBoxFuture<'a, ()> { up_link_settle::settle(self, name, iters) }
    pub fn park_until_done<'a>(&'a mut self, name: &'a str, done: &'a Done) -> LocalBoxFuture<'a, ()> { up_link_park_until_done::park_until_done(self, name, done) }
}

// no test_usage necessary
