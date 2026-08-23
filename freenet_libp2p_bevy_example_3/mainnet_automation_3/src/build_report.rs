use crate::instance;
use crate::outcome;

pub fn build_report(
    contract_params: &str,
    run_dir: &str,
    instances: &[instance::Instance],
    outcome: &outcome::Outcome,
    killed: bool,
) -> String {
    let mut s =
        format!("<b>local-mainnet run</b> · contract={contract_params}\nrun-dir: {run_dir}\n");
    for (inst, o) in instances.iter().zip(&outcome.instances) {
        s.push_str(&format!(
            "  instance-{} pid={} moved={} peers={} ready={}\n   log: {}\n",
            inst.index,
            inst.pid,
            o.moved,
            o.applied_peer_ids,
            o.ready,
            inst.log_path.display()
        ));
    }
    let race = if outcome.put_count > 1 {
        format!("⚠ RACE: {}-Put", outcome.put_count)
    } else {
        format!("clean seed ({} Put)", outcome.put_count)
    };
    s.push_str(&format!(
        "roster seed: {race}\nflap: max cumulative offline {:.1}s\n",
        outcome.max_offline_secs
    ));
    if outcome.error_sigs.is_empty() {
        s.push_str("no error signatures\n");
    } else {
        s.push_str("error signatures:\n");
        for e in &outcome.error_sigs {
            s.push_str(&format!("  {e}\n"));
        }
    }
    s.push_str(&format!(
        "killed: {killed} · pgrep {}\n",
        if killed {
            "clean"
        } else {
            "UNEXPECTED leftovers"
        }
    ));
    s.push_str("grep: -E \"error|panic|timed out|sending roster Put\" instance-*/app.log\n");
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::build_report;
    use crate::instance_outcome;
    use crate::outcome;

    #[test]
    fn test_usage() {
        let report = build_report(
            "p",
            "/run",
            &[],
            &outcome::Outcome {
                instances: vec![instance_outcome::InstanceOutcome {
                    moved: false,
                    applied_peer_ids: 0,
                    ready: false,
                }],
                put_count: 0,
                error_sigs: vec![],
                max_offline_secs: 0.0,
            },
            true,
        );
        assert!(report.contains("local-mainnet run"));
    }
}
