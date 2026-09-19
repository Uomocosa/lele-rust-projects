use super::chaos::Chaos;
use super::network_profile::NetworkProfile;
use super::topology::Topology;

#[must_use]
pub fn with_seed(profile: &NetworkProfile, seed: u64) -> NetworkProfile {
    NetworkProfile {
        name: profile.name,
        seed,
        min_latency: profile.min_latency,
        max_latency: profile.max_latency,
        fail_rate: profile.fail_rate,
        tcp_capacity: profile.tcp_capacity,
        dial_within: profile.dial_within,
        redial_every: profile.redial_every,
        stagger: profile.stagger,
        link_down_after: profile.link_down_after,
        slow_pairs: profile.slow_pairs.clone(),
        topology: match profile.topology {
            Topology::Star { center } => Topology::Star { center },
            Topology::FullMesh => Topology::FullMesh,
        },
        nat: profile.nat,
        ghosts: profile.ghosts,
        chaos: match &profile.chaos {
            Chaos::Calm => Chaos::Calm,
            Chaos::Leave { who, at } => Chaos::Leave { who, at: *at },
            Chaos::Flaky { a, b, cycles } => Chaos::Flaky {
                a,
                b,
                cycles: *cycles,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::Chaos;
    use super::NetworkProfile;
    use super::Topology;
    use super::with_seed;

    #[test]
    fn test_usage() {
        let profile = NetworkProfile {
            name: "lan",
            seed: 7,
            min_latency: std::time::Duration::from_millis(5),
            max_latency: std::time::Duration::from_millis(20),
            fail_rate: 0.0,
            tcp_capacity: 1024,
            dial_within: std::time::Duration::from_secs(5),
            redial_every: std::time::Duration::from_secs(5),
            stagger: std::time::Duration::ZERO,
            link_down_after: std::time::Duration::from_secs(5),
            slow_pairs: Vec::new(),
            topology: Topology::FullMesh,
            nat: super::super::Nat::Open,
            ghosts: &[],
            chaos: Chaos::Calm,
        };
        assert_eq!(with_seed(&profile, 8).seed, 8);
    }
}
