use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Page<T> {
    pub total_count: usize,
    pub total_page: usize,
    pub page_size: usize,
    pub cur_page: usize,

    pub data: Vec<T>
}