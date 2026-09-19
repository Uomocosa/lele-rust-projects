use std::time::Duration;

use clicker_lib::{testing, testing::turmoil as rig};
use futures_util::future::LocalBoxFuture;
use rstest::rstest;

struct PeerTask {
    plan: rig::DialPlan,
    slot: usize,
    results: rig::Results,
    done: rig::Done,
}

fn run_peer(task: PeerTask) -> LocalBoxFuture<'static, turmoil::Result> {
    Box::pin(async move {
        let PeerTask {
            plan,
            slot,
            results,
            done,
        } = task;
        let name = plan.name;
        let clicks = plan.clicks;
        let mut link = rig::bring_up(plan).await?;
        link.handshake(name).await;
        testing::click_times(&mut link.app, clicks);
        let want = testing::MeshCount::of([
            (testing::Player(1), 1),
            (testing::Player(2), 5),
            (testing::Player(3), 17),
        ]);
        link.converge_to(name, want).await;
        if let Some(entry) = results.borrow_mut().get_mut(slot) {
            *entry = Some(rig::counts_of(&mut link.app));
        }
        link.park_until_done(name, &done).await;
        Ok(())
    })
}

fn scenario(seed: u64) -> turmoil::Result {
    let profile = rig::with_seed(&rig::FLEET[0], seed);
    let results: rig::Results = std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let mut sim = rig::build_sim(&profile);
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let shared = results.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, &profile);
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                results: shared.clone(),
                done: parked.clone(),
            })
        });
    }
    let reported = results.clone();
    let parked = done;
    sim.client("client", async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        turmoil::hold("peer-3", "peer-1");
        turmoil::hold("peer-3", "peer-2");
        tokio::time::sleep(Duration::from_secs(8)).await;
        turmoil::release("peer-3", "peer-1");
        turmoil::release("peer-3", "peer-2");
        let settled = tokio::time::timeout(Duration::from_secs(120), async {
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
        *parked.borrow_mut() = true;
        Ok(())
    });
    sim.run()?;
    let want = testing::MeshCount::of([
        (testing::Player(1), 1),
        (testing::Player(2), 5),
        (testing::Player(3), 17),
    ]);
    let results = results.borrow();
    for (slot, got) in results.iter().enumerate() {
        if got.as_ref() != Some(&want) {
            return Err(format!("peer-{} diverged", slot.saturating_add(1)).into());
        }
    }
    Ok(())
}

#[rstest]
fn asymmetric_1_5_17_heals_after_hold(#[values(42, 43)] seed: u64) {
    assert!(scenario(seed).is_ok(), "turmoil scenario failed");
}
