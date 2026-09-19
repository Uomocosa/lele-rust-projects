use std::time::Duration;

use super::network_profile::NetworkProfile;

#[must_use]
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

// no test_usage necessary
