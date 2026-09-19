use std::time::Duration;

use clicker_lib::{testing, testing::turmoil as rig};
use futures_util::future::LocalBoxFuture;
use rstest::rstest;

const LEAVE_AT: Duration = Duration::from_secs(30);
const SETTLE_ITERS: u32 = 300;
const EXPECTED_TOTAL: i32 = 23;

struct PeerTask {
    plan: rig::DialPlan,
    slot: usize,
    leave: bool,
    results: rig::Results,
    gone: rig::Gone,
    done: rig::Done,
}

fn run_peer(task: PeerTask) -> LocalBoxFuture<'static, turmoil::Result> {
    Box::pin(async move {
        let PeerTask {
            plan,
            slot,
            leave,
            results,
            gone,
            done,
        } = task;
        let name = plan.name;
        let clicks = plan.clicks;
        let mut link = rig::bring_up(plan).await?;
        link.handshake(name).await;
        testing::click_times(&mut link.app, clicks);
        if leave {
            let start = tokio::time::Instant::now();
            while start.elapsed() < LEAVE_AT {
                link.pump(name).await;
                tokio::time::sleep(rig::STEP_SLEEP).await;
            }
            if let Some(entry) = gone.borrow_mut().get_mut(slot) {
                *entry = true;
            }
            return Ok(());
        }
        let want = testing::MeshCount::of([
            (testing::Player(1), 1),
            (testing::Player(2), 5),
            (testing::Player(3), 17),
        ]);
        for _ in 0..rig::CONVERGE_ITERS {
            link.pump(name).await;
            if rig::counts_of(&mut link.app) == want
                && gone.borrow().get(2).copied().unwrap_or(false)
            {
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
    let gone: rig::Gone = std::rc::Rc::new(std::cell::RefCell::new(vec![false, false, false]));
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    let mut sim = rig::build_sim(profile);
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let seen = results.clone();
        let missing = gone.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, profile);
        let leave = lane.name == "peer-3";
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                leave,
                results: seen.clone(),
                gone: missing.clone(),
                done: parked.clone(),
            })
        });
    }
    let reported = results.clone();
    let missing = gone;
    let parked = done;
    sim.client("client", async move {
        let settled = tokio::time::timeout(Duration::from_secs(300), async {
            loop {
                let ready = {
                    let seen = reported.borrow();
                    let missing = missing.borrow();
                    seen.first().is_some_and(std::option::Option::is_some)
                        && seen.get(1).is_some_and(std::option::Option::is_some)
                        && missing.get(2).copied().unwrap_or(false)
                };
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
    sim.run()?;
    let results = results.borrow();
    for (slot, got) in results.iter().enumerate().take(2) {
        let global = got.as_ref().map_or(-1, |counts| counts.global);
        if global != EXPECTED_TOTAL {
            return Err(format!(
                "survivor peer-{} lost the leaver's total",
                slot.saturating_add(1)
            )
            .into());
        }
    }
    Ok(())
}

#[rstest]
#[case::leave_lan(&rig::FLEET[3])]
#[case::leave_relay(&rig::LEAVE_RELAY)]
fn total_retained_after_leave(#[case] profile: &rig::NetworkProfile) {
    assert!(scenario(profile).is_ok(), "turmoil scenario failed");
}

#[rstest]
fn total_retained_seed_matrix(#[values(7, 8, 9)] seed: u64) {
    let profile = rig::with_seed(&rig::FLEET[3], seed);
    assert!(scenario(&profile).is_ok(), "turmoil scenario failed");
}
