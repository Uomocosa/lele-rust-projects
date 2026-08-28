use crate::Error;
use crate::bridge_counts;
use crate::collect_error_sigs;
use crate::final_count;
use crate::has_put;
use crate::instance;
use crate::instance_outcome;
use crate::is_ready;
use crate::outcome;

pub fn evaluate(
    instances: &[instance::Instance],
    observation_secs: u64,
) -> Result<outcome::Outcome, Error> {
    let mut outs = Vec::with_capacity(instances.len());
    for inst in instances {
        let ready = is_ready::is_ready(&inst.log_path)?;
        let count = final_count::final_count(&inst.log_path)?;
        let put = has_put::has_put(&inst.log_path)?;
        outs.push(instance_outcome::InstanceOutcome {
            ready,
            final_count: count,
            put,
        });
    }

    let put_count = outs.iter().filter(|o| o.put).count();

    let all_ready = outs.iter().all(|o| o.ready);
    let counts: Vec<u64> = outs.iter().filter_map(|o| o.final_count).collect();
    let converged = all_ready && !counts.is_empty() && counts.iter().all(|c| *c == counts[0]);
    let aggregated = if converged {
        let c = counts[0];
        c as f64 / observation_secs.max(1) as f64 >= 1.5
    } else {
        false
    };

    let log_refs: Vec<&std::path::Path> = instances.iter().map(|i| i.log_path.as_path()).collect();
    let error_sigs = collect_error_sigs::collect_error_sigs(&log_refs)?;
    let (bridge_splits, bridge_merges) = bridge_counts::bridge_counts(&log_refs)?;

    Ok(outcome::Outcome {
        instances: outs,
        put_count,
        converged,
        aggregated,
        error_sigs,
        bridge_splits,
        bridge_merges,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::evaluate;
    use crate::instance;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("e2e_eval_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let tent = dir.join("instance-0").join("app.log");
        fs::create_dir_all(dir.join("instance-0")).unwrap();
        fs::write(
            &tent,
             "connected, running indefinitely\n\
              tick count=5 owns=5\n\
              tick count=6 owns=6\n\
              contract deployed\n\
              bridge: split suspected\n\
              bridge: merged via resubscribe\n",
        )
        .unwrap();
        let inst = instance::Instance {
            index: 0,
            pid: 1,
            title: "t".to_string(),
            log_path: tent,
        };
        let outcome = evaluate(&[inst], 2).unwrap();
        assert!(outcome.instances[0].ready);
        assert_eq!(outcome.instances[0].final_count, Some(6));
        assert!(outcome.instances[0].put);
        assert_eq!(outcome.put_count, 1);
        assert_eq!(outcome.bridge_splits, 1);
        assert_eq!(outcome.bridge_merges, 1);
        assert!(outcome.error_sigs.is_empty());
    }
}
