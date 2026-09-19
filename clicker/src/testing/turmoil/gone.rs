use std::cell::RefCell;
use std::rc::Rc;

pub type Gone = Rc<RefCell<Vec<bool>>>;

// no test_usage necessary
