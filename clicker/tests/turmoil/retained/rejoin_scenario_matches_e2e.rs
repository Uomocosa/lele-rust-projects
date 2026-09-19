use std::time::Duration;

use clicker_lib::testing;
use clicker_lib::testing::scenario::Step;
use clicker_lib::testing::scenario::rejoin_scenario;
use clicker_lib::testing::turmoil as rig;
use futures_util::future::LocalBoxFuture;

const CLICKS: u32 = 15;
const RUN_ITERS: u32 = 16000;
const SETTLE_ITERS: u32 = 400;
const EXPECTED_TOTAL: i32 = 45;

struct PeerTask {
    plan: rig::DialPlan,
    slot: usize,
    results: rig::Results,
    healed: rig::Gone,
    done: rig::Done,
}

fn own_player(name: &str) -> u64 {
    name.strip_prefix("peer-")
        .and_then(|rest| rest.parse::<u64>().ok())
        .unwrap_or_default()
}

fn run_peer(task: PeerTask) -> LocalBoxFuture<'static, turmoil::Result> {
    Box::pin(async move {
        let PeerTask {
            plan,
            slot,
            results,
            healed,
            done,
        } = task;
        let name = plan.name;
        let player = own_player(name);
        let mut link = rig::bring_up(plan).await?;
        link.handshake(name).await;
        for step in rejoin_scenario(CLICKS) {
            if let Step::Click {
                player: target,
                times,
            } = step
                && target == player
            {
                testing::click_times(&mut link.app, times);
            }
        }
        for _ in 0..RUN_ITERS {
            link.pump(name).await;
            if healed.borrow().get(slot).copied().unwrap_or(false) {
                link.settle(name, SETTLE_ITERS).await;
                break;
            }
            tokio::time::sleep(rig::STEP_SLEEP).await;
        }
        if let Some(entry) = results.borrow_mut().get_mut(slot) {
            *entry = Some(rig::counts_of(&mut link.app));
        }
        link.park_until_done(name, &done).await;
        Ok(())
    })
}

fn scenario(profile: &rig::NetworkProfile) -> turmoil::Result {
    let results: rig::Results = std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let healed: rig::Gone = std::rc::Rc::new(std::cell::RefCell::new(vec![false, false, false]));
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    let mut sim = rig::build_sim(profile);
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let seen = results.clone();
        let fixed = healed.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, profile);
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                results: seen.clone(),
                healed: fixed.clone(),
                done: parked.clone(),
            })
        });
    }
    let reported = results.clone();
    sim.client("client", async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        turmoil::partition("peer-2", "peer-1");
        turmoil::partition("peer-2", "peer-3");
        tokio::time::sleep(Duration::from_secs(5)).await;
        turmoil::repair("peer-2", "peer-1");
        turmoil::repair("peer-2", "peer-3");
        tokio::time::sleep(Duration::from_secs(10)).await;
        for entry in healed.borrow_mut().iter_mut() {
            *entry = true;
        }
        let settled = tokio::time::timeout(Duration::from_secs(200), async {
            loop {
                if reported.borrow().iter().all(std::option::Option::is_some) {
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
        *done.borrow_mut() = true;
        Ok(())
    });
    sim.run()?;
    for (slot, got) in results.borrow().iter().enumerate() {
        let global = got.as_ref().map_or(-1, |counts| counts.global);
        if global != EXPECTED_TOTAL {
            return Err(format!(
                "peer-{} global {global} != {EXPECTED_TOTAL}",
                slot.saturating_add(1)
            )
            .into());
        }
    }
    Ok(())
}

#[rstest::rstest]
fn rejoin_scenario_matches_e2e(#[values(7, 8)] seed: u64) {
    let profile = rig::with_seed(&rig::FLEET[0], seed);
    assert!(scenario(&profile).is_ok(), "turmoil scenario failed");
}
