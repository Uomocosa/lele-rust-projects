use clicker_lib::{clicker, lobby};
use freenet_libp2p_bevy_plugin::{p2p, roster};

#[test]
fn join_emits_fetch_roster_single_round_trip() {
    let mut active = clicker::ActiveLobby::default();
    let mut selected = lobby::SelectedRoom::default();
    let mut roster_lobby = roster::Lobby::default();
    let mut pending = lobby::JoinPending::default();
    let mut gate = lobby::JoinGate::default();
    let mut clock = lobby::JoinClock::default();
    let mut commands = p2p::Commands::<clicker::CursorMsg>::default();
    lobby::join_room(
        "alpha",
        &mut lobby::JoinRooms {
            active: &mut active,
            selected: &mut selected,
            roster_lobby: &mut roster_lobby,
            pending: &mut pending,
            gate: &mut gate,
            clock: &mut clock,
        },
        &mut commands,
    );
    assert!(
        commands.iter().any(|command| matches!(
            command,
            p2p::Command::FetchRoster { lobby } if lobby == "alpha"
        )),
        "join sends exactly one FetchRoster discovery round-trip"
    );
    assert_eq!(*pending, Some("alpha".to_string()));
}
