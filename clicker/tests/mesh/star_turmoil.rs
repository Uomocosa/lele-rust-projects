use std::time::Duration;

use clicker_lib::testing;
use rstest::rstest;

use crate::net_profile;
use crate::turmoil_rig;

fn run_peer(
    name: &'static str,
    own: u64,
    clicks: u32,
    dial: &'static [&'static str],
    accept: usize,
    slot: usize,
    link_down_after: Duration,
    results: turmoil_rig::Results,
    done: turmoil_rig::Done,
) -> impl std::future::Future<Output = turmoil::Result> + 'static {
    async move {
        let mut link = turmoil_rig::bring_up(name, own, dial, accept, link_down_after).await?;
        turmoil_rig::handshake(&mut link, name).await;
        testing::click_times(&mut link.app, clicks);
        let want = testing::MeshCount::wanted(1, 5, 17);
        turmoil_rig::converge_to(&mut link, name, want).await;
        results.borrow_mut()[slot] = Some(turmoil_rig::counts_of(&mut link.app));
        turmoil_rig::park_until_done(&mut link, name, &done).await;
        Ok(())
    }
}

fn scenario(seed: u64) {
    let profile = net_profile::with_seed(&net_profile::FLEET[1], seed);
    let results: turmoil_rig::Results =
        std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let mut sim = net_profile::build_sim(&profile);
    let done: turmoil_rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    for (slot, lane) in net_profile::lanes_for(&profile.topology)
        .into_iter()
        .enumerate()
    {
        let seen = results.clone();
        let parked = done.clone();
        let (name, own, clicks, dial, accept) =
            (lane.name, lane.own, lane.clicks, lane.dial, lane.accept);
        let link_down_after = profile.link_down_after;
        sim.host(name, move || {
            run_peer(
                name,
                own,
                clicks,
                dial,
                accept,
                slot,
                link_down_after,
                seen.clone(),
                parked.clone(),
            )
        });
    }
    let seen = results.clone();
    let parked = done.clone();
    sim.client("client", async move {
        let settled = tokio::time::timeout(Duration::from_secs(180), async {
            loop {
                if seen.borrow().iter().all(|r| r.is_some()) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;
        if settled.is_err() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::TimedOut, "peers never reported").into(),
            );
        }
        *parked.borrow_mut() = true;
        Ok(())
    });
    sim.run().expect("turmoil sim");
    let want = testing::MeshCount::wanted(1, 5, 17);
    let results = results.borrow();
    for (slot, got) in results.iter().enumerate() {
        assert_eq!(*got, Some(want), "peer-{} diverged", slot.saturating_add(1));
    }
}

#[rstest]
fn star_leaf_to_leaf_heals_via_center(#[values(7, 8, 9)] seed: u64) {
    scenario(seed);
}
