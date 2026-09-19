use std::time::Duration;

use clicker_lib::{testing, testing::turmoil as rig};
use futures_util::future::LocalBoxFuture;

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

#[test]
fn cold_start_stagger_converges() {
    let profile = rig::COLD;
    let results: rig::Results = std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let mut sim = rig::build_sim(&profile);
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let seen = results.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, &profile);
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                results: seen.clone(),
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
            "peer-{} failed to converge under cold-start stagger",
            slot.saturating_add(1)
        );
    }
}
