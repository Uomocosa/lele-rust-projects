use crate::instance;
use crate::outcome;

pub fn build_report(
    contract_params: &str,
    run_dir: &str,
    instances: &[instance::Instance],
    outcome: &outcome::Outcome,
    killed: bool,
) -> String {
    let mut s = format!(
        "<b>freenet_example mainnet e2e</b> · contract={contract_params}\nrun-dir: {run_dir}\n"
    );
    for (inst, o) in instances.iter().zip(&outcome.instances) {
        let count = o
            .final_count
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        s.push_str(&format!(
            "  instance-{} pid={} ready={} put={} final_count={count}\n   log: {}\n",
            inst.index,
            inst.pid,
            o.ready,
            o.put,
            inst.log_path.display()
        ));
    }
    let put = if outcome.put_count > 1 {
        format!(
            "⚠ multi-Put race window: {:?} instances seeded",
            outcome.put_count
        )
    } else {
        format!("single Put seed ({} instance)", outcome.put_count)
    };
    s.push_str(&format!("seeding: {put}\n"));
    let converge = match (outcome.converged, outcome.aggregated) {
        (true, true) => "converged AND aggregated (all see each other)".to_string(),
        (true, false) => {
            "converged but NOT aggregated (equal counts at ~1× rate ⇒ split replicas / trivial)"
                .to_string()
        }
        (false, _) => "NOT converged (counters diverged)".to_string(),
    };
    s.push_str(&format!("convergence: {converge}\n"));
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
    s.push_str("grep: -E \"contract deployed|count=|panic|error\" instance-*/app.log\n");
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
                    ready: true,
                    final_count: Some(9),
                    put: true,
                }],
                put_count: 1,
                converged: true,
                aggregated: true,
                error_sigs: vec![],
            },
            true,
        );
        assert!(report.contains("freenet_example mainnet e2e"));
    }
}
