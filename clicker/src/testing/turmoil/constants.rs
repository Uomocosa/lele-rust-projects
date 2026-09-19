use std::time::Duration;

use super::chaos::Chaos;
use super::ghost_hint::GhostHint;
use super::nat::Nat;
use super::network_profile::NetworkProfile;
use super::topology::Topology;

pub const PORT: u16 = 17381;
pub const HANDSHAKE_ITERS: u32 = 300;
pub const CONVERGE_ITERS: u32 = 2000;
pub const STEP_SLEEP: Duration = Duration::from_millis(5);
pub const MAX_FRAME: usize = 4_000_000;

pub const FLEET: [NetworkProfile; 4] = [
    NetworkProfile {
        name: "lan",
        seed: 7,
        min_latency: Duration::from_millis(5),
        max_latency: Duration::from_millis(20),
        fail_rate: 0.0,
        tcp_capacity: 1024,
        dial_within: Duration::from_secs(5),
        redial_every: Duration::from_secs(5),
        stagger: Duration::ZERO,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        nat: Nat::Open,
        ghosts: &[],
        chaos: Chaos::Calm,
    },
    NetworkProfile {
        name: "star-relay",
        seed: 7,
        min_latency: Duration::from_millis(5),
        max_latency: Duration::from_millis(20),
        fail_rate: 0.0,
        tcp_capacity: 64,
        dial_within: Duration::from_secs(5),
        redial_every: Duration::from_secs(5),
        stagger: Duration::ZERO,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::Star { center: "peer-1" },
        nat: Nat::Open,
        ghosts: &[],
        chaos: Chaos::Calm,
    },
    NetworkProfile {
        name: "flaky-nat",
        seed: 7,
        min_latency: Duration::from_millis(20),
        max_latency: Duration::from_millis(300),
        fail_rate: 0.02,
        tcp_capacity: 64,
        dial_within: Duration::from_secs(15),
        redial_every: Duration::from_secs(5),
        stagger: Duration::ZERO,
        link_down_after: Duration::from_secs(15),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        nat: Nat::Open,
        ghosts: &[],
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
        dial_within: Duration::from_secs(5),
        redial_every: Duration::from_secs(5),
        stagger: Duration::ZERO,
        link_down_after: Duration::from_secs(5),
        slow_pairs: Vec::new(),
        topology: Topology::FullMesh,
        nat: Nat::Open,
        ghosts: &[],
        chaos: Chaos::Leave {
            who: "peer-3",
            at: Duration::from_secs(30),
        },
    },
];

pub const LEAVE_RELAY: NetworkProfile = NetworkProfile {
    name: "leave-relay",
    seed: 7,
    min_latency: Duration::from_millis(5),
    max_latency: Duration::from_millis(20),
    fail_rate: 0.0,
    tcp_capacity: 64,
    dial_within: Duration::from_secs(5),
    redial_every: Duration::from_secs(5),
    stagger: Duration::ZERO,
    link_down_after: Duration::from_secs(5),
    slow_pairs: Vec::new(),
    topology: Topology::Star { center: "peer-1" },
    nat: Nat::Open,
    ghosts: &[],
    chaos: Chaos::Leave {
        who: "peer-3",
        at: Duration::from_secs(30),
    },
};

pub const GHOST: NetworkProfile = NetworkProfile {
    name: "ghost",
    seed: 7,
    min_latency: Duration::from_millis(5),
    max_latency: Duration::from_millis(20),
    fail_rate: 0.0,
    tcp_capacity: 1024,
    dial_within: Duration::from_secs(2),
    redial_every: Duration::from_secs(5),
    stagger: Duration::ZERO,
    link_down_after: Duration::from_secs(5),
    slow_pairs: Vec::new(),
    topology: Topology::FullMesh,
    nat: Nat::Open,
    ghosts: &[GhostHint {
        name: "peer-9",
        port: 9,
    }],
    chaos: Chaos::Calm,
};

pub const COLD: NetworkProfile = NetworkProfile {
    name: "cold",
    seed: 7,
    min_latency: Duration::from_millis(5),
    max_latency: Duration::from_millis(20),
    fail_rate: 0.0,
    tcp_capacity: 1024,
    dial_within: Duration::from_secs(3),
    redial_every: Duration::from_secs(1),
    stagger: Duration::from_secs(1),
    link_down_after: Duration::from_secs(5),
    slow_pairs: Vec::new(),
    topology: Topology::FullMesh,
    nat: Nat::Open,
    ghosts: &[],
    chaos: Chaos::Calm,
};

pub const HAIRPIN: NetworkProfile = NetworkProfile {
    name: "hairpin",
    seed: 7,
    min_latency: Duration::from_millis(5),
    max_latency: Duration::from_millis(20),
    fail_rate: 0.0,
    tcp_capacity: 1024,
    dial_within: Duration::from_secs(2),
    redial_every: Duration::from_secs(5),
    stagger: Duration::ZERO,
    link_down_after: Duration::from_secs(5),
    slow_pairs: Vec::new(),
    topology: Topology::FullMesh,
    nat: Nat::NoHairpin {
        public_ip: "10.0.0.1",
    },
    ghosts: &[],
    chaos: Chaos::Calm,
};
