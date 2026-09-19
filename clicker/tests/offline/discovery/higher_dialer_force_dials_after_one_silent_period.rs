use clicker_lib::discovery;

#[test]
fn higher_dialer_force_dials_after_one_silent_period() {
    let patient = discovery::decide_dial("peer-2", "peer-1", Some(discovery::REDIAL_SECS - 1));
    assert_eq!(patient, discovery::DialDecision::Wait);
    let desperate = discovery::decide_dial("peer-2", "peer-1", Some(discovery::REDIAL_SECS));
    assert_eq!(desperate, discovery::DialDecision::ForceDial);
}
