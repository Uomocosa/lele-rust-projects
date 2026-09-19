use std::collections::HashMap;
use std::collections::HashSet;

use bevy::prelude::App;
use freenet_libp2p_bevy_plugin::p2p;
use futures_util::future::LocalBoxFuture;

use super::accept_loop::accept_loop;
use super::conn_writer::ConnWriter;
use super::constants::PORT;
use super::dial_plan::DialPlan;
use super::dial_with_deadline::dial_with_deadline;
use super::dialable::dialable;
use super::envelope::Envelope;
use super::link_app::link_app;
use super::public_ip::public_ip;
use super::register::register;
use super::up_link::UpLink;
use super::write_hello::write_hello;
use crate::clicker;
use crate::testing;

pub fn bring_up(plan: DialPlan) -> LocalBoxFuture<'static, turmoil::Result<UpLink>> {
    Box::pin(async move {
        let DialPlan {
            name,
            own,
            clicks: _,
            dial,
            accept,
            ghosts,
            public_ip: own_ip,
            nat,
            link_down_after,
            dial_within,
            listen_at,
            redial_every,
        } = plan;
        tokio::time::sleep(listen_at).await;
        let mut app = testing::fixture(own, "alpha");
        link_app(&mut app, name);
        let listener = turmoil::net::TcpListener::bind(("0.0.0.0", PORT)).await?;
        let (accept_tx, mut accept_rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(accept_loop(listener, accept_tx.clone()));
        let (inbox_tx, inbox_rx) = tokio::sync::mpsc::unbounded_channel::<Envelope>();
        let mut outbound: HashMap<String, ConnWriter> = HashMap::new();
        let mut dial_failed: Vec<String> = Vec::new();
        let mut redial_targets: Vec<&'static str> = Vec::new();

        for target in dial.iter().copied() {
            let target_ip = public_ip(nat, target);
            if !dialable(nat, own_ip, target_ip) {
                dial_failed.push(target.to_string());
                push_failed(&mut app, target, "no-hairpin");
                continue;
            }
            redial_targets.push(target);
            match dial_with_deadline(target, PORT, dial_within).await {
                Ok(mut stream) => {
                    if write_hello(&mut stream, name).await.is_err() {
                        dial_failed.push(target.to_string());
                        push_failed(&mut app, target, "hello");
                        continue;
                    }
                    register(&mut app, &mut outbound, &inbox_tx, target, stream);
                }
                Err(reason) => {
                    dial_failed.push(target.to_string());
                    push_failed(&mut app, target, reason);
                }
            }
        }

        for ghost in ghosts {
            if let Err(reason) = dial_with_deadline(ghost.name, ghost.port, dial_within).await {
                dial_failed.push(ghost.name.to_string());
                push_failed(&mut app, ghost.name, reason);
            }
        }

        let expected = dial.len().saturating_add(accept);
        let accept_start = tokio::time::Instant::now();
        while outbound.len() < expected {
            let remaining = dial_within.saturating_sub(accept_start.elapsed());
            if remaining.is_zero() {
                break;
            }
            let Ok(Some((origin, stream))) =
                tokio::time::timeout(remaining, accept_rx.recv()).await
            else {
                break;
            };
            if !outbound.contains_key(&origin) {
                register(&mut app, &mut outbound, &inbox_tx, &origin, stream);
            }
        }

        Ok(UpLink {
            app,
            outbound,
            inbox_tx,
            inbox_rx,
            accept_rx,
            accept_tx,
            seen: HashSet::new(),
            seq: 0,
            dead: HashSet::new(),
            last_sync: HashMap::new(),
            link_down_after,
            dial_failed,
            redial_targets,
            redial_every,
            last_redial: HashMap::new(),
        })
    })
}

// needed helper: surfaces a failed dial as an app event
fn push_failed(app: &mut App, peer: &str, reason: &str) {
    app.world_mut()
        .resource_mut::<p2p::Events<clicker::CursorMsg>>()
        .push(p2p::Event::DialFailed {
            peer_id: peer.to_string(),
            reason: reason.to_string(),
        });
}

// no test_usage necessary
