use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub enum Tier {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "bronze")]
    Bronze,
    #[serde(rename = "silver")]
    Silver,
    #[serde(rename = "gold")]
    Gold,
    #[serde(rename = "diamond")]
    Diamond,
}
