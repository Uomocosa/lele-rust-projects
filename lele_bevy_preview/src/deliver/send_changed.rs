use crate::Error;
use crate::deliver;

pub fn send_changed(
    captures: &[deliver::Capture],
    clip: Option<&deliver::Capture>,
) -> Result<usize, Error> {
    let mut sent: usize = 0;
    for capture in captures.iter().chain(clip) {
        if !deliver::sendable::sendable(capture.status) {
            continue;
        }
        let caption = format!(
            "{} · {} · {:?}",
            capture.artifact.scene, capture.artifact.label, capture.status
        );
        if deliver::send_one::send_one(&capture.path, &caption)? {
            sent = sent.saturating_add(1);
        }
    }
    Ok(sent)
}
// no test_usage necessary
