use std::cell::RefCell;
use std::rc::Rc;

use crate::testing;

pub type Results = Rc<RefCell<Vec<Option<testing::MeshCount>>>>;

// no test_usage necessary
