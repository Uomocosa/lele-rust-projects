use freenet_stdlib::prelude::UpdateData;

pub fn decode_update(data: UpdateData<'static>) -> Option<Vec<u8>> {
    match data {
        UpdateData::State(state) => Some(state.as_ref().to_vec()),
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        UpdateData::StateAndDelta { state, .. } => Some(state.as_ref().to_vec()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::prelude::{State, StateDelta, UpdateData};

    use super::decode_update;

    #[test]
    fn test_usage() {
        let update = UpdateData::State(State::from(vec![1, 2, 3]));
        assert_eq!(decode_update(update).as_deref(), Some(&[1, 2, 3][..]));

        let delta = UpdateData::Delta(StateDelta::from(vec![9]));
        assert_eq!(decode_update(delta).as_deref(), Some(&[9][..]));
    }
}
