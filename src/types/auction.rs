use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{enums::*, types::*};
use uuid::Uuid;
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Auction {
    pub id: String,
    pub minimal_reputation: i32,
    #[serde(rename = "winner")]
    pub winner_id: Option<String>,
    pub private: bool,
    pub visible: bool,
    pub note_raw: String,
    pub note: String,
    pub starting_price: i32,
    pub buyout_price: Option<i32>,
    pub is_direct_sell: bool,
    pub top_bid: Option<i32>,
    pub created: String,
    pub updated: String,
    pub platform: String,
    pub closed: bool,
    pub is_marked_for: Option<String>,
    pub marked_operation_at: Option<String>,
    pub item: AuctionItem,
    #[serde(default)]
    pub uuid: String,
    pub properties: Option<serde_json::Value>, // Additional properties for the order
}
impl Auction {
    pub fn apply_uuid(&mut self) {
        if self.uuid.is_empty() {
            self.uuid = self.item.uuid().to_string();
        }
    }
    pub fn set_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuctionWithOwner {
    #[serde(flatten)]
    pub auction: Auction,

    pub owner: UserShort,
}
impl AuctionWithOwner {
    pub fn apply_uuid(&mut self) {
        self.auction.apply_uuid();
    }
}
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct AuctionItem {
    #[serde(rename = "type")]
    pub item_type: AuctionType,

    pub weapon_url_name: String,

    // RIVEN
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub mod_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<ItemAttribute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re_rolls: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mastery_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_rank: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polarity: Option<String>,

    // SISTER / LICH
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quirk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub having_ephemera: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,

    // Similarity information for the item Is not from WFM
    #[serde(default)]
    pub similarity: Similarity,
}
impl AuctionItem {
    /// Generate a UUID based on all fields + attributes
    pub fn uuid(&self) -> Uuid {
        let mut input = String::new();

        input.push_str(&format!("type:{};", self.item_type as i32));
        input.push_str(&format!("weapon:{};", self.weapon_url_name));

        if let Some(v) = &self.mod_name {
            input.push_str(&format!("mod_name:{};", v.to_lowercase()));
        }
        if let Some(v) = &self.re_rolls {
            input.push_str(&format!("re_rolls:{};", v));
        }
        if let Some(v) = &self.mastery_level {
            input.push_str(&format!("mastery:{};", v));
        }
        if let Some(v) = &self.mod_rank {
            input.push_str(&format!("mod_rank:{};", v));
        }
        if let Some(v) = &self.polarity {
            input.push_str(&format!("polarity:{};", v.to_lowercase()));
        }
        if let Some(v) = &self.quirk {
            input.push_str(&format!("quirk:{};", v.to_lowercase()));
        }
        if let Some(v) = &self.element {
            input.push_str(&format!("element:{};", v.to_lowercase()));
        }
        if let Some(v) = &self.having_ephemera {
            input.push_str(&format!("ephemera:{};", v));
        }
        if let Some(v) = &self.damage {
            input.push_str(&format!("damage:{};", v));
        }
        if let Some(attrs) = &self.attributes {
            // Sort attributes by URL name
            let mut sorted_attrs = attrs.clone();
            sorted_attrs.sort_by_key(|a| a.url_name.clone());
            for a in sorted_attrs {
                input.push_str(&format!("attr:{}:{}:{};", a.url_name, a.positive, a.value));
            }
        }
        Uuid::new_v5(&Uuid::NAMESPACE_OID, input.as_bytes())
    }

    /// Compare this auction item's attributes (candidate) against the provided `attributes` (reference/base).
    /// - missing: in base but not in this item
    /// - extra:   in this item but not in base
    pub fn apply_similarity(&mut self, attributes: Vec<ItemAttribute>) -> Similarity {
        if self.item_type != AuctionType::Riven {
            return Similarity::default();
        }

        let cand = self.attributes.clone().unwrap_or_default(); // this auction's attributes
        let base_set: HashSet<(String, bool)> = attributes
            .iter()
            .map(|a| (a.url_name.clone(), a.positive))
            .collect();
        let cand_set: HashSet<(String, bool)> = cand
            .iter()
            .map(|a| (a.url_name.clone(), a.positive))
            .collect();

        // missing = base \ cand
        let mut missing: Vec<String> = base_set
            .difference(&cand_set)
            .map(|(name, pos)| format!("{}:{}", name, pos))
            .collect();

        // extra = cand \ base
        let mut extra: Vec<String> = cand_set
            .difference(&base_set)
            .map(|(name, pos)| format!("{}:{}", name, pos))
            .collect();

        // Deterministic order
        missing.sort();
        extra.sort();

        // Jaccard similarity over unique keys
        let intersection = base_set.intersection(&cand_set).count() as f32;
        let union = base_set.union(&cand_set).count() as f32;
        let score = if union > 0.0 {
            intersection / union
        } else {
            -1.0
        };

        let similarity = Similarity {
            score,
            missing,
            extra,
        };
        self.similarity = similarity.clone();
        similarity
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemAttribute {
    pub url_name: String,
    pub positive: bool,
    pub value: f64,
    #[serde(default = "unknown_effect")] // Is set by the user
    pub effect: String,

    pub properties: Option<serde_json::Value>, // Additional properties for the order
}

fn unknown_effect() -> String {
    String::from("Unknown Effect")
}

impl ItemAttribute {
    pub fn new(url_name: impl Into<String>, positive: bool, value: f64) -> Self {
        Self {
            url_name: url_name.into(),
            positive,
            value,
            effect: String::from("Unknown Effect"),
            properties: None,
        }
    }
    pub fn set_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
}
