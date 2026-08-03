// no test_usage necessary
pub fn print_checker_list(checkers: &[Box<dyn crate::checker::Checker>]) {
    for c in checkers {
        println!("{:>5}  {}", c.code(), c.name());
    }
}
