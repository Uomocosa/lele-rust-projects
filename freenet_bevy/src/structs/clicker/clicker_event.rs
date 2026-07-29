use freenet_stdlib::prelude::ContractKey;

#[derive(Debug)]
pub enum ClickerEvent {
    Init {
        contract_key: ContractKey,
        count: u64,
    },
    Notification {
        count: u64,
    },
    UpdateResponse {
        count: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::ClickerEvent;
    use freenet_stdlib::prelude::ContractKey;

    #[test]
    fn test_usage() {
        let key = ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let evt = ClickerEvent::Init {
            contract_key: key,
            count: 5,
        };
        match evt {
            ClickerEvent::Init {
                contract_key,
                count,
            } => {
                assert_eq!(contract_key, key);
                assert_eq!(count, 5);
            }
            _ => panic!("expected Init"),
        }
    }
}
