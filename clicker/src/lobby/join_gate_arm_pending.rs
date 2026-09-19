use crate::lobby;

pub fn arm_pending(gate: &mut lobby::JoinGate, entry: lobby::PendingJoin) {
    if let Some(existing) = gate
        .pending
        .iter_mut()
        .find(|held| held.joiner == entry.joiner)
    {
        *existing = entry;
        return;
    }
    gate.pending.push(entry);
}

#[cfg(test)]
mod tests {
    use super::arm_pending;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    fn entry(reveal_at: std::time::Instant) -> lobby::PendingJoin {
        lobby::PendingJoin {
            peer: "peer-2".to_string(),
            joiner: net_id::NetworkId(2),
            reveal_at,
        }
    }

    #[test]
    fn test_usage() {
        let now = std::time::Instant::now();
        let mut gate = lobby::JoinGate::default();
        arm_pending(&mut gate, entry(now));
        assert_eq!(gate.pending.len(), 1);
        let later = now
            .checked_add(std::time::Duration::from_secs(5))
            .unwrap_or(now);
        arm_pending(&mut gate, entry(later));
        assert_eq!(
            gate.pending.len(),
            1,
            "same joiner re-armed, not duplicated"
        );
        assert_eq!(gate.pending.first().map(|held| held.reveal_at), Some(later));
    }
}
