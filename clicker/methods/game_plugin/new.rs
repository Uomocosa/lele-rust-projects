use crate::clicker;

#[must_use]
pub const fn new() -> clicker::GamePlugin {
    clicker::GamePlugin
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let _plugin = new();
    }
}
