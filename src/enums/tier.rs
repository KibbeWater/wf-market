use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub enum Tier {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "premium")]
    Premium,
    #[serde(rename = "vip")]
    Vip,
}