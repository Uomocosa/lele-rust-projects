use super::dial_plan::DialPlan;
use super::lane::Lane;
use super::network_profile::NetworkProfile;
use super::own_of::own_of;
use super::public_ip::public_ip;

#[must_use]
pub fn plan_for(lane: &Lane, profile: &NetworkProfile) -> DialPlan {
    let own = lane.own;
    let dial = lane
        .visible
        .iter()
        .copied()
        .filter(|peer| own_of(peer) > own)
        .collect();
    let accept = lane
        .visible
        .iter()
        .filter(|peer| own_of(peer) < own)
        .count();
    DialPlan {
        name: lane.name,
        own,
        clicks: lane.clicks,
        dial,
        accept,
        ghosts: profile.ghosts,
        public_ip: public_ip(profile.nat, lane.name),
        nat: profile.nat,
        link_down_after: profile.link_down_after,
        dial_within: profile.dial_within,
        redial_every: profile.redial_every,
        listen_at: profile.stagger.mul_f64(f64::from(
            u32::try_from(own.saturating_sub(1)).unwrap_or(u32::MAX),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::Chaos;
    use super::super::Nat;
    use super::super::Topology;
    use super::DialPlan;
    use super::Lane;
    use super::NetworkProfile;
    use super::plan_for;

    fn profile() -> NetworkProfile {
        NetworkProfile {
            name: "t",
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
        }
    }

    #[test]
    fn test_usage() {
        let p = profile();
        let p1 = Lane {
            name: "peer-1",
            own: 1,
            clicks: 1,
            visible: &["peer-2", "peer-3"],
        };
        let plan: DialPlan = plan_for(&p1, &p);
        assert_eq!(plan.dial, vec!["peer-2", "peer-3"]);
        assert_eq!(plan.accept, 0);

        let p3 = Lane {
            name: "peer-3",
            own: 3,
            clicks: 17,
            visible: &["peer-1", "peer-2"],
        };
        let plan = plan_for(&p3, &p);
        assert_eq!(plan.dial, Vec::<&str>::new());
        assert_eq!(plan.accept, 2);
    }
}
