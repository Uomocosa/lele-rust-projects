use std::time::Duration;

use clicker_lib::testing;
use rstest::rstest;

use crate::net_profile;
use crate::turmoil_rig;

const LEAVE_AT: Duration = Duration::from_secs(30);
const SETTLE_ITERS: u32 = 300;
const EXPECTED_TOTAL: i32 = 23;

fn run_peer(
    name: &'static str,
    own: u64,
    clicks: u32,
    dial: &'static [&'static str],
    accept: usize,
    slot: usize,
    leave: bool,
    results: turmoil_rig::Results,
    gone: turmoil_rig::Gone,
    done: turmoil_rig::Done,
) -> impl std::future::Future<Output = turmoil::Result> + 'static {
    async move {
        let mut link = turmoil_rig::bring_up(name, own, dial, accept).await?;
        turmoil_rig::handshake(&mut link, name).await;
        testing::click_times(&mut link.app, clicks);
        if leave {
            // A real process keeps syncing until it dies: pump while waiting
            // so the room converges before the kill. Bare sleep would drop
            // unsent clicks with the sockets and test nothing.
            let start = tokio::time::Instant::now();
            while start.elapsed() < LEAVE_AT {
                turmoil_rig::pump(
                    &mut link.app,
                    name,
                    &mut link.outbound,
                    &mut link.inbox_rx,
                    &mut link.seen,
                    &mut link.seq,
                    &mut link.dead,
                )
                .await;
                tokio::time::sleep(turmoil_rig::STEP_SLEEP).await;
            }
            gone.borrow_mut()[slot] = true;
            return Ok(());
        }
        let want = testing::MeshCount::wanted(1, 5, 17);
        for _ in 0..turmoil_rig::CONVERGE_ITERS {
            turmoil_rig::pump(
                &mut link.app,
                name,
                &mut link.outbound,
                &mut link.inbox_rx,
                &mut link.seen,
                &mut link.seq,
                &mut link.dead,
            )
            .await;
            if turmoil_rig::counts_of(&mut link.app) == want && gone.borrow()[2] {
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
    let mut sim = net_profile::build_sim(profile);
    for (slot, lane) in net_profile::lanes_for(&profile.topology)
        .into_iter()
        .enumerate()
    {
        let seen = results.clone();
        let missing = gone.clone();
        let parked = done.clone();
        let (name, own, clicks, dial, accept) =
            (lane.name, lane.own, lane.clicks, lane.dial, lane.accept);
        let leave = name == "peer-3";
        sim.host(name, move || {
            run_peer(
                name,
                own,
                clicks,
                dial,
                accept,
                slot,
                leave,
                seen.clone(),
                missing.clone(),
                parked.clone(),
            )
        });
    }
    let seen = results.clone();
    let missing = gone.clone();
    let parked = done.clone();
    sim.client("client", async move {
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
            "survivor peer-{} lost the leaver's total",
            slot.saturating_add(1)
        );
    }
}

#[rstest]
#[case::leave_lan(&net_profile::FLEET[3])]
#[case::leave_relay(&net_profile::LEAVE_RELAY)]
fn total_retained_after_leave(#[case] profile: &net_profile::NetworkProfile) {
    scenario(profile);
}
