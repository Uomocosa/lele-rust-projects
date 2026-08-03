pub fn load_data() -> Vec<u8> {
    vec![1, 2, 3]
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = super::load_data();
    }
}
