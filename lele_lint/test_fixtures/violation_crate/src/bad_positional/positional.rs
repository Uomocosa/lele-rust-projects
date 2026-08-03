use super::positional_new;

pub struct Positional(pub String, pub u32);

impl Default for Positional {
    fn default() -> Self {
        Self(String::new(), 0)
    }
}

#[rustfmt::skip]
impl Positional {
    pub fn new() -> Self { positional_new::new() }
}

#[cfg(test)]
mod tests {
    use super::Positional;

    #[test]
    fn test_usage() {
        let p = Positional::new();
        let _name = p.0;             // VIOLATION: positional field access
        let _count = p.1;            // VIOLATION: positional field access
    }
}
