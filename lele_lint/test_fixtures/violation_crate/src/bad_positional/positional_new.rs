use super::positional::Positional;

pub fn new() -> Positional {
    Positional::default()
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let _p = new();
    }
}
