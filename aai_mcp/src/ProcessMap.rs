use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::ProcessHandle;

pub type ProcessMap = Arc<Mutex<HashMap<u32, ProcessHandle>>>;
