use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Page<T> {
    pub total: i64,
    pub records: Vec<T>
}