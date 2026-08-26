use crate::trial_result;

pub fn build_report(trials: &[trial_result::TrialResult], instances: usize) -> String {
    let mut s = format!(
        "<b>freenet_example_2 · mainnet reconcile e2e</b>\ninstances/run: {instances} · trials: {}\n",
        trials.len()
    );
    for t in trials {
        let counts = t
            .final_counts
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let latency = t
            .latency_secs
            .map(|l| format!("{l}s"))
            .unwrap_or("-".into());
        let union = t
            .expected_union
            .map(|u| u.to_string())
            .unwrap_or_else(|| "-".into());
        let merged = t
            .merged_correct
            .map(|m| m.to_string())
            .unwrap_or_else(|| "-".into());
        let status = if !t.ready {
            "✗ not-all-connected"
        } else if t.reconciled && t.merged_correct.unwrap_or(t.aggregated) {
            "✓ reconciled+merged"
        } else if t.reconciled {
            "~ reconciled (not merged)"
        } else {
            "✗ NOT reconciled"
        };
        s.push_str(&format!(
            "  <b>{}-r{}</b> {status} put={} final=[{counts}] latency={latency} union={union} merged={merged} errs={}\n",
            t.mode, t.rep, t.put_count, t.error_sigs.len()
        ));
    }

    for mode in ["counter", "set"] {
        let ms: Vec<&trial_result::TrialResult> =
            trials.iter().filter(|t| t.mode == mode).collect();
        if ms.is_empty() {
            continue;
        }
        let rec = ms.iter().filter(|t| t.reconciled).count();
        let lat: Vec<u64> = ms.iter().filter_map(|t| t.latency_secs).collect();
        let avg = if lat.is_empty() {
            "-".to_string()
        } else {
            format!("{:.0}s", lat.iter().sum::<u64>() / lat.len() as u64)
        };
        s.push_str(&format!(
            "  <b>{mode}</b>: reconciled {rec}/{} · avg latency {avg}\n",
            ms.len()
        ));
    }
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::build_report;

    #[test]
    fn test_usage() {
        let report = build_report(&[], 2);
        assert!(report.contains("reconcile e2e"));
    }
}
