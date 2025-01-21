use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub type Database = Arc<Mutex<HashMap<String, String>>>;

pub fn init_db() -> Database {
    Arc::new(Mutex::new(HashMap::new()))
}
