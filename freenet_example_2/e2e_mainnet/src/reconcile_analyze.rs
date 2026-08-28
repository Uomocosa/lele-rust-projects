use std::collections::BTreeSet;

use crate::reconcile_result;
use crate::tick_sample;

// needed helper:
fn tolerance(instances: usize) -> u64 {
    instances as u64 + 3
}

pub fn analyze(
    traces: &[Vec<tick_sample::TickSample>],
    mode: &str,
) -> reconcile_result::ReconcileResult {
    let tol = tolerance(traces.len());
    let final_counts: Vec<u64> = traces
        .iter()
        .filter_map(|t| t.last().map(|s| s.count))
        .collect();

    let all_final = final_counts.len() == traces.len() && !final_counts.is_empty();
    let reconciled = all_final
        && final_counts
            .iter()
            .all(|c| *c >= final_counts[0].saturating_sub(tol) && *c <= final_counts[0] + tol);

    let end = traces
        .iter()
        .filter_map(|t| t.last().map(|s| s.secs))
        .max()
        .unwrap_or(0);
    let start = traces
        .iter()
        .filter_map(|t| t.first().map(|s| s.secs))
        .min()
        .unwrap_or(0);
    let last_div = last_divergence_secs(traces, tol);
    let latency = if reconciled {
        Some(end.saturating_sub(last_div))
    } else {
        None
    };

    let span = end.saturating_sub(start).max(1);
    let aggregated = reconciled && final_counts[0] as f64 / span as f64 >= 1.5;

    let delta_mergeable = mode == "set" || mode == "counter";
    let expected_union = if delta_mergeable {
        Some(traces.iter().filter_map(|t| t.last().map(|s| s.owns)).sum())
    } else {
        None
    };
    let merged_correct = if delta_mergeable {
        let eu = expected_union.unwrap_or(0);
        Some(reconciled && final_counts[0] + tol >= eu && final_counts[0] <= eu + tol)
    } else {
        None
    };

    reconcile_result::ReconcileResult {
        final_counts,
        reconciled,
        latency_secs: latency,
        expected_union,
        merged_correct,
        aggregated,
    }
}

// needed helper:
fn last_divergence_secs(traces: &[Vec<tick_sample::TickSample>], tol: u64) -> u64 {
    let mut secs: BTreeSet<u64> = BTreeSet::new();
    for t in traces {
        for s in t {
            secs.insert(s.secs);
        }
    }
    let mut last_div = 0;
    for sec in secs {
        let vals: Vec<u64> = traces
            .iter()
            .map(|t| {
                t.iter()
                    .take_while(|s| s.secs <= sec)
                    .last()
                    .map(|s| s.count)
                    .unwrap_or(0)
            })
            .collect();
        let lo = *vals.iter().min().unwrap_or(&0);
        let hi = *vals.iter().max().unwrap_or(&0);
        if hi.saturating_sub(lo) > tol {
            last_div = sec;
        }
    }
    last_div
}

#[cfg(test)]
mod tests {
    use super::analyze;
    use crate::tick_sample::TickSample;

    fn t(secs: u64, count: u64, owns: u64) -> TickSample {
        TickSample { secs, count, owns }
    }

    #[test]
    fn test_usage() {
        let a = vec![t(100, 1, 1), t(101, 2, 2), t(102, 3, 3), t(103, 7, 4)];
        let b = vec![t(100, 1, 1), t(101, 2, 2), t(103, 7, 3)];
        let r = analyze(&[a, b], "set");
        assert!(r.reconciled);
        assert_eq!(r.expected_union, Some(7));
        assert_eq!(r.merged_correct, Some(true));
    }

    #[test]
    fn test_usage_counter_merges() {
        let a = vec![t(100, 1, 1), t(101, 2, 2), t(103, 10, 7)];
        let b = vec![t(100, 1, 1), t(101, 2, 2), t(103, 10, 3)];
        let r = analyze(&[a, b], "counter");
        assert!(r.reconciled);
        assert_eq!(r.expected_union, Some(10));
        assert_eq!(r.merged_correct, Some(true));
    }

    #[test]
    fn test_usage_split_not_merged() {
        let a = vec![t(100, 0, 0), t(200, 50, 50)];
        let b = vec![t(100, 0, 0), t(200, 50, 50)];
        let r = analyze(&[a, b], "set");
        assert!(r.reconciled);
        assert_eq!(r.expected_union, Some(100));
        assert_eq!(r.merged_correct, Some(false));
    }

    #[test]
    fn test_usage_tolerance_scales_with_instances() {
        assert_eq!(super::tolerance(2), 5);
        assert_eq!(super::tolerance(3), 6);
        assert_eq!(super::tolerance(5), 8);
    }
}
