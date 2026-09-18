use std::time::Duration;

use clicker_lib::testing;

use crate::net_profile;
use crate::turmoil_rig;

const PARTITION_AT: Duration = Duration::from_secs(15);
const LEAVE_ITERS: u32 = 8000;
const EXTRA_AT_ITER: u32 = 2000;
const EXTRA_CLICKS: u32 = 11;
const SETTLE_ITERS: u32 = 300;
const EXPECTED_TOTAL: i32 = 34;

fn run_peer(
    name: &'static str,
    own: u64,
    clicks: u32,
    dial: &'static [&'static str],
    accept: usize,
    slot: usize,
    leave: bool,
    link_down_after: Duration,
    results: turmoil_rig::Results,
    gone: turmoil_rig::Gone,
    repaired: turmoil_rig::Gone,
    done: turmoil_rig::Done,
) -> impl std::future::Future<Output = turmoil::Result> + 'static {
    async move {
        let mut link = turmoil_rig::bring_up(name, own, dial, accept, link_down_after).await?;
        turmoil_rig::handshake(&mut link, name).await;
        testing::click_times(&mut link.app, clicks);
        if leave {
            for iter in 0..LEAVE_ITERS {
                if iter == EXTRA_AT_ITER {
                    testing::click_times(&mut link.app, EXTRA_CLICKS);
                }
                turmoil_rig::pump(
                    &mut link.app,
                    name,
                    &mut link.outbound,
                    &mut link.inbox_rx,
                    &mut link.seen,
                    &mut link.seq,
                    &mut link.dead,
                    &mut link.last_sync,
                    link.link_down_after,
                )
                .await;
                tokio::time::sleep(turmoil_rig::STEP_SLEEP).await;
            }
            gone.borrow_mut()[slot] = true;
            return Ok(());
        }
        for _ in 0..16000 {
            turmoil_rig::pump(
                &mut link.app,
                name,
                &mut link.outbound,
                &mut link.inbox_rx,
                &mut link.seen,
                &mut link.seq,
                &mut link.dead,
                &mut link.last_sync,
                link.link_down_after,
            )
            .await;
            if gone.borrow()[2] && repaired.borrow()[2] {
                turmoil_rig::settle(&mut link, name, SETTLE_ITERS).await;
                break;
            }
            tokio::time::sleep(turmoil_rig::STEP_SLEEP).await;
        }
        results.borrow_mut()[slot] = Some(turmoil_rig::counts_of(&mut link.app));
        turmoil_rig::park_until_done(&mut link, name, &done).await;
        Ok(())
    }
}

fn scenario(profile: &net_profile::NetworkProfile) {
    let results: turmoil_rig::Results =
        std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let gone: turmoil_rig::Gone =
        std::rc::Rc::new(std::cell::RefCell::new(vec![false, false, false]));
    let done: turmoil_rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    let healed: turmoil_rig::Gone =
        std::rc::Rc::new(std::cell::RefCell::new(vec![false, false, false]));
    let mut sim = net_profile::build_sim(profile);
    for (slot, lane) in net_profile::lanes_for(&profile.topology)
        .into_iter()
        .enumerate()
    {
        let seen = results.clone();
        let missing = gone.clone();
        let fixed = healed.clone();
        let parked = done.clone();
        let (name, own, clicks, dial, accept) =
            (lane.name, lane.own, lane.clicks, lane.dial, lane.accept);
        let leave = name == "peer-3";
        let link_down_after = profile.link_down_after;
        sim.host(name, move || {
            run_peer(
                name,
                own,
                clicks,
                dial,
                accept,
                slot,
                leave,
                link_down_after,
                seen.clone(),
                missing.clone(),
                fixed.clone(),
                parked.clone(),
            )
        });
    }
    let seen = results.clone();
    let missing = gone.clone();
    let fixed = healed.clone();
    let parked = done.clone();
    sim.client("client", async move {
        tokio::time::sleep(PARTITION_AT).await;
        turmoil::partition("peer-2", "peer-1");
        turmoil::partition("peer-2", "peer-3");
        let left = tokio::time::timeout(Duration::from_secs(500), async {
            loop {
                if missing.borrow()[2] {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;
        if left.is_err() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::TimedOut, "leaver never left").into(),
            );
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
        turmoil::repair("peer-2", "peer-1");
        turmoil::repair("peer-2", "peer-3");
        tokio::time::sleep(Duration::from_secs(10)).await;
        fixed.borrow_mut()[2] = true;
        let settled = tokio::time::timeout(Duration::from_secs(300), async {
            loop {
                let ready =
                    seen.borrow()[0].is_some() && seen.borrow()[1].is_some() && missing.borrow()[2];
                if ready {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;
        if settled.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "survivors never reported",
            )
            .into());
        }
        *parked.borrow_mut() = true;
        Ok(())
    });
    sim.run().expect("turmoil sim");
    let results = results.borrow();
    for (slot, got) in results.iter().enumerate().take(2) {
        let global = got.map(|counts| counts.global).unwrap_or(-1);
        assert_eq!(
            global,
            EXPECTED_TOTAL,
            "diverged survivor peer-{} never healed after repair",
            slot.saturating_add(1)
        );
    }
}

#[test]
fn diverged_survivors_heal_after_repair() {
    scenario(&net_profile::FLEET[3]);
}
