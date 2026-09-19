use clicker_lib::discovery;

#[test]
fn simultaneous_learn_triggers_single_dialer() {
    let lower = discovery::decide_dial("peer-1", "peer-2", None);
    let higher = discovery::decide_dial("peer-2", "peer-1", None);
    assert_eq!(lower, discovery::DialDecision::Dial);
    assert_eq!(higher, discovery::DialDecision::Wait);
}
