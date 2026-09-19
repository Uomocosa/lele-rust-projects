use std::time::Duration;

use clicker_lib::testing::turmoil as rig;
use futures_util::future::LocalBoxFuture;

type Report = std::rc::Rc<std::cell::RefCell<Vec<Option<(usize, Vec<String>)>>>>;

struct PeerTask {
    plan: rig::DialPlan,
    slot: usize,
    report: Report,
    done: rig::Done,
}

fn run_peer(task: PeerTask) -> LocalBoxFuture<'static, turmoil::Result> {
    Box::pin(async move {
        let PeerTask {
            plan,
            slot,
            report,
            done,
        } = task;
        let name = plan.name;
        let mut link = rig::bring_up(plan).await?;
        if let Some(entry) = report.borrow_mut().get_mut(slot) {
            *entry = Some((link.outbound.len(), link.dial_failed.clone()));
        }
        link.park_until_done(name, &done).await;
        Ok(())
    })
}

#[test]
fn hairpin_blocks_same_public_ip() {
    let profile = rig::HAIRPIN;
    let report: Report = std::rc::Rc::new(std::cell::RefCell::new(vec![None, None, None]));
    let done: rig::Done = std::rc::Rc::new(std::cell::RefCell::new(false));
    let mut sim = rig::build_sim(&profile);
    for (slot, lane) in rig::lanes_for(&profile.topology).into_iter().enumerate() {
        let seen = report.clone();
        let parked = done.clone();
        let plan = rig::plan_for(&lane, &profile);
        sim.host(lane.name, move || {
            run_peer(PeerTask {
                plan: plan.clone(),
                slot,
                report: seen.clone(),
                done: parked.clone(),
            })
        });
    }
    let seen = report.clone();
    let parked = done;
    sim.client("client", async move {
        let settled = tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                if seen.borrow().iter().all(std::option::Option::is_some) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
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
    let report = report.borrow();
    for (slot, entry) in report.iter().enumerate() {
        let (outbound, _) = entry.as_ref().expect("report written");
        assert_eq!(
            *outbound,
            0,
            "peer-{} must not connect under no-hairpin",
            slot.saturating_add(1)
        );
    }
    let p1 = &report[0].as_ref().expect("p1").1;
    assert!(
        p1.contains(&"peer-2".to_string()) && p1.contains(&"peer-3".to_string()),
        "hairpin dials must surface DialFailed, got {p1:?}"
    );
}
