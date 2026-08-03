pub fn report(msg: &str) {
    println!("report: {}", msg);              // VIOLATION: println! instead of tracing
    eprintln!("ERROR: {}", msg);              // VIOLATION: eprintln! instead of tracing
    dbg!(msg);                                // VIOLATION: dbg! instead of tracing
}

#[cfg(test)]
mod tests {
    use super::report;

    #[test]
    fn test_usage() {
        report("test");
    }
}
