use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiErrorBody {
    pub request: Option<Vec<String>>,
    pub inputs: Option<HashMap<String, String>>,
}
