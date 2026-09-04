use freenet_stdlib::prelude::UpdateData;

#[must_use]
pub fn update_data_bytes(update: &UpdateData<'_>) -> Option<Vec<u8>> {
    match update {
        UpdateData::State(s) => Some(s.as_ref().to_vec()),
        UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(update_data_bytes);
    }
}
