use crate::Checker;

pub fn print_checker_list(checkers: &[Box<dyn Checker>]) {
    for c in checkers {
        println!("{:>5}  {}", c.code(), c.name());
    }
}

// no test_usage necessary
