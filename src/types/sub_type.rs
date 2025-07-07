use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>, // Subtype of the item, if applicable

    // MODS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>, // Rank of the mod, if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charges: Option<u8>, // Charges remaining (Requiem mods)

    // AYATAN SCULPTURES
    #[serde(rename = "amberStars", skip_serializing_if = "Option::is_none")]
    pub amber_stars: Option<u8>, // Number of Amber Stars, if applicable
    #[serde(rename = "cyanStars", skip_serializing_if = "Option::is_none")]
    pub cyan_stars: Option<u8>, // Number of Cyan Stars, if applicable
}
impl Default for SubType {
    fn default() -> Self {
        SubType {
            subtype: None,
            rank: None,
            charges: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }
}
impl Hash for SubType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rank.hash(state);
        self.subtype.hash(state);
        self.amber_stars.hash(state);
        self.cyan_stars.hash(state);
    }
}
impl SubType {
    pub fn parazon_mod(charges: u8) -> Self {
        SubType {
            subtype: None,
            rank: None,
            charges: Some(charges),
            amber_stars: None,
            cyan_stars: None,
        }
    }
    pub fn mods(rank: u8) -> Self {
        SubType {
            subtype: None,
            rank: Some(rank),
            charges: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }
    pub fn ayatan_sculpture(amber_stars: u8, cyan_stars: u8) -> Self {
        SubType {
            subtype: None,
            rank: None,
            charges: None,
            amber_stars: Some(amber_stars),
            cyan_stars: Some(cyan_stars),
        }
    }
    pub fn variant(subtype: &str) -> Self {
        SubType {
            subtype: Some(subtype.to_string()),
            rank: None,
            charges: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }
}
impl Display for SubType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(rank) = self.rank {
            write!(f, "Rank: {} ", rank)?;
        }
        if let Some(subtype) = &self.subtype {
            write!(f, "Subtype: {} ", subtype)?;
        }
        if let Some(amber_stars) = self.amber_stars {
            write!(f, "Amber Stars: {} ", amber_stars)?;
        }
        if let Some(cyan_stars) = self.cyan_stars {
            write!(f, "Cyan Stars: {} ", cyan_stars)?;
        }
        Ok(())
    }
}
