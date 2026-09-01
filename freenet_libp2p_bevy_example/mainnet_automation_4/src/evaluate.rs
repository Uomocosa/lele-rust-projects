use crate::Error;
use crate::applied_player_ids;
use crate::collect_error_sigs;
use crate::has_roster_put;
use crate::instance;
use crate::instance_outcome;
use crate::is_ready;
use crate::max_cumulative_offline_secs;
use crate::outcome;
use crate::snapshot_x_range;

const MOVE_THRESHOLD: f64 = 20.0;

pub fn evaluate(instances: &[instance::Instance]) -> Result<outcome::Outcome, Error> {
    let mut instance_outcomes = Vec::with_capacity(instances.len());
    for inst in instances {
        let ready = is_ready::is_ready(&inst.log_path)?;
        let (min, max) = snapshot_x_range::snapshot_x_range(&inst.log_path)?;
        let moved = ready && (max - min) > MOVE_THRESHOLD;
        let peers = applied_player_ids::applied_player_ids(&inst.log_path)?;
        instance_outcomes.push(instance_outcome::InstanceOutcome {
            moved,
            applied_peer_ids: peers.len(),
            ready,
        });
    }
    let mut put_count = 0usize;
    for inst in instances {
        if has_roster_put::has_roster_put(&inst.log_path)? {
            put_count += 1;
        }
    }
    let log_refs: Vec<&std::path::Path> = instances.iter().map(|i| i.log_path.as_path()).collect();
    let error_sigs = collect_error_sigs::collect_error_sigs(&log_refs)?;
    let mut max_offline_secs = 0.0_f64;
    for inst in instances {
        max_offline_secs = max_offline_secs.max(
            max_cumulative_offline_secs::max_cumulative_offline_secs(&inst.log_path)?,
        );
    }
    Ok(outcome::Outcome {
        instances: instance_outcomes,
        put_count,
        error_sigs,
        max_offline_secs,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::evaluate;
    use crate::instance;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("ma_eval_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("app.log");
        fs::write(
            &log,
            format!(
                "embedded node ready\n\
                 sending engine snapshot player_id=3 x=0.0\n\
                 sending engine snapshot player_id=3 x=120.0\n\
                 received peer input player_id={}\n",
                "ab".repeat(32)
            ),
        )
        .unwrap();
        let inst = instance::Instance {
            index: 0,
            pid: 1,
            log_path: log.clone(),
            identity_dir: dir.clone(),
        };
        let outcome = evaluate(&[inst]).unwrap();
        assert!(outcome.instances[0].ready);
        assert!(outcome.instances[0].moved);
        assert_eq!(outcome.instances[0].applied_peer_ids, 1);
        assert!(outcome.error_sigs.is_empty());
    }
}
