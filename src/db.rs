use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub type Database = Arc<Mutex<HashMap<String, Data>>>;

pub struct Data {
    pub creation_data: DateTime<Local>,
    pub shortened_url: String,
    pub long_url: String,
    pub ttl: u8,
}

pub fn init_db() -> Database {
    Arc::new(Mutex::new(HashMap::new()))
}
