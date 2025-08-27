use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Similarity {
    pub score: f32,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

impl Default for Similarity {
    fn default() -> Self {
        Similarity {
            score: -9999.0,
            missing: Vec::new(),
            extra: Vec::new(),
        }
    }
}
