use serde::{Deserialize, Serialize};

use crate::types::ItemAttribute;

#[derive(Hash, Eq, PartialEq)]
pub struct AttrKey {
    pub name: String,
    pub positive: bool,
}
impl From<&ItemAttribute> for AttrKey {
    fn from(attr: &ItemAttribute) -> Self {
        AttrKey {
            name: attr.url_name.clone(),
            positive: attr.positive,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Similarity {
    pub score: f32,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}
impl Similarity {
    pub fn has_attribute(&self, url: impl Into<String>) -> bool {
        let url = url.into();
        self.missing.iter().any(|a| a.starts_with(&url))
            || self.extra.iter().any(|a| a.starts_with(&url))
    }
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
