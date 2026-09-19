use crate::lobby;

pub fn take_due_pending(
    gate: &mut lobby::JoinGate,
    now: std::time::Instant,
) -> Vec<lobby::PendingJoin> {
    let mut due = Vec::new();
    let mut rest = Vec::new();
    for entry in gate.pending.drain(..) {
        if entry.reveal_at <= now {
            due.push(entry);
        } else {
            rest.push(entry);
        }
    }
    gate.pending.extend(rest);
    due
}

#[cfg(test)]
mod tests {
    use super::take_due_pending;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    fn entry(joiner: u64, reveal_at: std::time::Instant) -> lobby::PendingJoin {
        lobby::PendingJoin {
            peer: format!("peer-{joiner}"),
            joiner: net_id::NetworkId(joiner),
            reveal_at,
        }
    }

    #[test]
    fn test_usage() {
        let now = std::time::Instant::now();
        let earlier = now
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or(now);
        let later = now
            .checked_add(std::time::Duration::from_secs(5))
            .unwrap_or(now);
        let mut gate = lobby::JoinGate::default();
        gate.pending.push(entry(2, earlier));
        gate.pending.push(entry(3, later));
        let due = take_due_pending(&mut gate, now);
        assert_eq!(due.len(), 1);
        assert_eq!(gate.pending.len(), 1, "not-yet-due entry retained");
    }
}
