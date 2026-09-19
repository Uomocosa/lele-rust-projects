use std::cell::RefCell;
use std::rc::Rc;

pub type Done = Rc<RefCell<bool>>;

// no test_usage necessary
