use serde::{Deserialize, Serialize};

pub type JsResult<T> = Result<T, JsError>;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsError {
    message: String,
}

impl JsError {
    pub fn new<E: ToString>(e: E) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}
