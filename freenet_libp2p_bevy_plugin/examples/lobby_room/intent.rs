#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::args;

#[derive(Resource)]
pub struct Intent {
    pub action: Option<args::Action>,
    pub sent: bool,
}

pub fn send_once(mut intent: ResMut<Intent>, mut commands: MessageWriter<discovery::Command>) {
    if intent.sent {
        return;
    }
    if let Some(action) = &intent.action {
        match action {
            args::Action::CreateRoom(room) => {
                commands.write(discovery::Command::Create(room.clone()));
                tracing::info!("lobby intent create room={}", room.as_str());
            }
            args::Action::JoinRoom(room) => {
                commands.write(discovery::Command::Join(room.clone()));
                tracing::info!("lobby intent join room={}", room.as_str());
            }
        }
    }
    intent.sent = true;
}

pub fn log_joined(intent: Res<Intent>, mut events: MessageReader<discovery::Event>) {
    for event in events.read() {
        if let discovery::Event::Joined(room) = event {
            match intent.action {
                Some(args::Action::CreateRoom(_)) => {
                    tracing::info!("lobby created room={}", room.as_str());
                }
                Some(args::Action::JoinRoom(_)) => {
                    tracing::info!("lobby joined room={}", room.as_str());
                }
                None => {}
            }
        }
    }
}
