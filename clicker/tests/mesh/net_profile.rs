use std::time::Duration;

pub const SEEDS: [u64; 3] = [7, 8, 9];

pub enum Topology {
    Star { center: &'static str },
    FullMesh,
}

pub enum Chaos {
    Calm,
    Leave {
        who: &'static str,
        at: Duration,
    },
    Flaky {
        a: &'static str,
        b: &'static str,
        cycles: u32,
    },
}

pub struct NetworkProfile {
    pub name: &'static str,
    pub seed: u64,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub fail_rate: f64,
    pub tcp_capacity: usize,
    pub link_down_after: Duration,
    pub slow_pairs: Vec<(&'static str, &'static str, Duration, f64)>,
    pub topology: Topology,
    pub chaos: Chaos,
}

pub struct Lane {
    pub name: &'static str,
    pub own: u64,
    pub clicks: u32,
    pub dial: &'static [&'static str],
    pub accept: usize,
}

const LANES: [(&'static str, u64, u32); 3] =
    [("peer-1", 1, 1), ("peer-2", 2, 5), ("peer-3", 3, 17)];

pub fn lanes_for(topology: &Topology) -> [Lane; 3] {
    let edges = |name: &'static str| -> (&'static [&'static str], usize) {
        match topology {
            Topology::FullMesh => match name {
                "peer-1" => (&["peer-2", "peer-3"], 2),
                "peer-2" => (&["peer-1", "peer-3"], 2),
                _ => (&["peer-1", "peer-2"], 2),
            },
            Topology::Star { center } => {
                if name == *center {
                    (&["peer-2", "peer-3"], 2)
                } else {
                    (&["peer-1"], 1)
                }
            }
        }
    };
    LANES.map(|(name, own, clicks)| {
        let (dial, accept) = edges(name);
        Lane {
            name,
            own,
            clicks,
            dial,
            accept,
        }
    })
}

pub fn build_sim(profile: &NetworkProfile) -> turmoil::Sim<'static> {
    let mut builder = turmoil::Builder::new();
    builder
        .rng_seed(profile.seed)
        .min_message_latency(profile.min_latency)
        .max_message_latency(profile.max_latency)
        .fail_rate(profile.fail_rate)
        .tcp_capacity(profile.tcp_capacity)
        .simulation_duration(Duration::from_secs(600));
    let mut sim = builder.build();
    for (a, b, latency, loss) in &profile.slow_pairs {
        sim.set_link_latency(*a, *b, *latency);
        sim.set_link_fail_rate(*a, *b, *loss);
    }
    sim
}

pub fn with_seed(profile: &NetworkProfile, seed: u64) -> NetworkProfile {
    NetworkProfile {
        name: profile.name,
        seed,
        min_latency: profile.min_latency,
        max_latency: profile.max_latency,
        fail_rate: profile.fail_rate,
        tcp_capacity: profile.tcp_capacity,
        link_down_after: profile.link_down_after,
        slow_pairs: profile.slow_pairs.clone(),
        topology: match profile.topology {
            Topology::Star { center } => Topology::Star { center },
            Topology::FullMesh => Topology::FullMesh,
        },
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

fn secs(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
}

fn millis(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

pub const FLEET: [NetworkProfile; 4] = [
    NetworkProfile {
        name: "lan",
        seed: 7,
        min_latency: Duration::from_millis(5),
        max_latency: Duration::from_millis(20),
        fail_rate: 0.0,
        tcp_capacity: 1024,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        chaos: Chaos::Calm,
    },
    NetworkProfile {
        name: "star-relay",
        seed: 7,
        min_latency: Duration::from_millis(5),
        max_latency: Duration::from_millis(20),
        fail_rate: 0.0,
        tcp_capacity: 64,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::Star { center: "peer-1" },
        chaos: Chaos::Calm,
    },
    NetworkProfile {
        name: "flaky-nat",
        seed: 7,
        min_latency: Duration::from_millis(20),
        max_latency: Duration::from_millis(300),
        fail_rate: 0.02,
        tcp_capacity: 64,
        link_down_after: Duration::from_secs(15),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        chaos: Chaos::Flaky {
            a: "peer-1",
            b: "peer-2",
            cycles: 3,
        },
    },
    NetworkProfile {
        name: "leave",
        seed: 7,
        min_latency: Duration::from_millis(5),
        max_latency: Duration::from_millis(50),
        fail_rate: 0.0,
        tcp_capacity: 1024,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        chaos: Chaos::Leave {
            who: "peer-3",
            at: Duration::from_secs(30),
        },
    },
];

#[allow(dead_code)]
pub fn relay_leave() -> NetworkProfile {
    with_seed(&FLEET[3], FLEET[3].seed)
}

pub const LEAVE_RELAY: NetworkProfile = NetworkProfile {
    name: "leave-relay",
    seed: 7,
    min_latency: Duration::from_millis(5),
    max_latency: Duration::from_millis(20),
    fail_rate: 0.0,
    tcp_capacity: 64,
    link_down_after: Duration::from_secs(5),
    slow_pairs: Vec::new(),
    topology: Topology::Star { center: "peer-1" },
    chaos: Chaos::Leave {
        who: "peer-3",
        at: Duration::from_secs(30),
    },
};

#[cfg(test)]
mod tests {
    use super::{FLEET, Topology, lanes_for};

    #[test]
    fn test_usage() {
        assert_eq!(FLEET.len(), 4);
        let lanes = lanes_for(&Topology::FullMesh);
        assert_eq!(lanes.len(), 3);
        assert_eq!(lanes[0].dial.len(), 2);
        let star = lanes_for(&Topology::Star { center: "peer-1" });
        assert_eq!(star[1].dial, &["peer-1"]);
        assert_eq!(star[0].accept, 2);
    }

    #[test]
    fn slow_pairs_compile() {
        let _ = super::with_seed(&FLEET[1], 8);
        let _ = super::millis(5);
        let _ = super::secs(1);
    }
}
