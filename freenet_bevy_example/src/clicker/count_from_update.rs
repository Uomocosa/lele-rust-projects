use freenet_stdlib::prelude::UpdateData;

pub fn count_from_update(update: &UpdateData) -> u64 {
    match update {
        UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
        UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::prelude::{State, UpdateData};

    use super::count_from_update;

    #[test]
    fn test_usage() {
        let state = bincode::serialize(&42u64).unwrap();
        let update = UpdateData::State(State::from(state));
        assert_eq!(count_from_update(&update), 42);
    }
}
