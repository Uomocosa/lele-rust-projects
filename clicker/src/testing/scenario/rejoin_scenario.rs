use super::step::Step;

#[must_use]
pub fn rejoin_scenario(clicks: u32) -> Vec<Step> {
    let total = i32::try_from(clicks.saturating_mul(3)).unwrap_or(i32::MAX);
    vec![
        Step::Click {
            player: 1,
            times: clicks,
        },
        Step::Click {
            player: 2,
            times: clicks,
        },
        Step::Click {
            player: 3,
            times: clicks,
        },
        Step::ExpectGlobal { total },
        Step::Partition {
            first: 2,
            second: 3,
        },
        Step::Heal {
            first: 2,
            second: 3,
        },
        Step::ExpectGlobal { total },
    ]
}

#[cfg(test)]
mod tests {
    use super::Step;
    use super::rejoin_scenario;

    #[test]
    fn test_usage() {
        let steps = rejoin_scenario(15);
        assert_eq!(steps.len(), 7);
        assert_eq!(
            steps.first().copied(),
            Some(Step::Click {
                player: 1,
                times: 15
            })
        );
        assert_eq!(
            steps.last().copied(),
            Some(Step::ExpectGlobal { total: 45 })
        );
    }
}
