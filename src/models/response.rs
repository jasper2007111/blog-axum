use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ResponseData<T> {
    pub msg: Option<String>,
    pub code: String,
    pub data: Option<T>,
}

impl<T> ResponseData<T> {
    pub fn of_success(data: T) -> Self {
        Self {
            msg: None,
            code: "10000".to_string(),
            data: Some(data),
        }
    }

    pub fn of_failure() -> Self {
        Self {
            msg: None,
            code: "60000".to_string(),
            data: None,
        }
    }
}
