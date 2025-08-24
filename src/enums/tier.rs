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

impl Tier {
    pub fn is_none(&self) -> bool {
        matches!(self, Tier::None)
    }

    pub fn is_premium(&self) -> bool {
        matches!(
            self,
            Tier::Bronze | Tier::Silver | Tier::Gold | Tier::Diamond
        )
    }
}
