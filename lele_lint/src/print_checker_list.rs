use crate::checker;

pub fn print_checker_list(checkers: &[Box<dyn checker::Checker>]) {
    for c in checkers {
        println!("{:>5}  {}", c.code(), c.name());
    }
}

// no test_usage necessary
