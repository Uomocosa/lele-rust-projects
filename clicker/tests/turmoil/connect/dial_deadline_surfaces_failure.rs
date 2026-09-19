use std::time::Duration;

use clicker_lib::{testing, testing::turmoil as rig};
use futures_util::future::LocalBoxFuture;

const SETTLE_ITERS: u32 = 200;

struct PeerTask {
    plan: rig::DialPlan,
    slot: usize,
    results: rig::Results,
    failures: std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>>,
    done: rig::Done,
}

fn run_peer(task: PeerTask) -> LocalBoxFuture<'static, turmoil::Result> {
    Box::pin(async move {
        let PeerTask {
            plan,
            slot,
            results,
            failures,
            done,
        } = task;
        let name = plan.name;
        let clicks = plan.clicks;
        let mut link = rig::bring_up(plan).await?;
        if let Some(entry) = failures.borrow_mut().get_mut(slot) {
            entry.clone_from(&link.dial_failed);
        }
        link.handshake(name).await;
        testing::click_times(&mut link.app, clicks);
        let want = testing::MeshCount::of([
            (testing::Player(1), 1),
            (testing::Player(2), 5),
            (testing::Player(3), 17),
        ]);
        link.converge_to(name, want).await;
        link.settle(name, SETTLE_ITERS).await;
        if let Some(entry) = results.borrow_mut().get_mut(slot) {
            *entry = Some(rig::counts_of(&mut link.app));
        }
        link.park_until_done(name, &done).await;
        Ok(())
    })
}

#[test]
fn dial_deadline_surfaces_failure() {
    let profile = rig::GHOST;
    let results: rig::Results = std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let failures: std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(vec![
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ]));
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    let mut sim = rig::build_sim(&profile);
    sim.host("peer-9", || async move { Ok(()) });
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let seen = results.clone();
        let failed = failures.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, &profile);
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                results: seen.clone(),
                failures: failed.clone(),
                done: parked.clone(),
            })
        });
    }
    let reported = results.clone();
    let parked = done;
    sim.client("client", async move {
        let settled = tokio::time::timeout(Duration::from_secs(180), async {
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
    sim.run().expect("turmoil sim");
    let want = testing::MeshCount::of([
        (testing::Player(1), 1),
        (testing::Player(2), 5),
        (testing::Player(3), 17),
    ]);
    for (slot, got) in results.borrow().iter().enumerate() {
        assert_eq!(
            got.as_ref(),
            Some(&want),
            "peer-{} failed to converge alongside the ghost",
            slot.saturating_add(1)
        );
    }
    let seen_failures = failures.borrow();
    assert!(
        seen_failures.iter().flatten().any(|f| f == "peer-9"),
        "ghost dial must surface a failed dial, got {seen_failures:?}"
    );
}
