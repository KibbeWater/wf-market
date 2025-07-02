use serde::Serialize;

use crate::{
    enums::{AuctionType, Polarity},
    types::ItemAttribute,
};

#[derive(Serialize)]
pub struct CreateAuctionParams {
    pub starting_price: i32,
    pub buyout_price: Option<i32>,
    pub minimal_reputation: i32,
    pub visible: bool,
    pub note: String,
    pub item: CreateAuctionItem,
}

impl CreateAuctionParams {
    pub fn new(
        starting_price: i32,
        buyout_price: Option<i32>,
        minimal_reputation: i32,
        visible: bool,
        note: &str,
        item: CreateAuctionItem,
    ) -> Self {
        Self {
            starting_price,
            buyout_price,
            minimal_reputation,
            visible,
            note: note.to_string(),
            item,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct CreateAuctionItem {
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
}
impl CreateAuctionItem {
    pub fn new_riven(
        weapon_url_name: &str,
        mod_name: &str,
        attributes: Vec<ItemAttribute>,
        re_rolls: i32,
        mastery_level: i32,
        mod_rank: i32,
        polarity: Polarity,
    ) -> Self {
        Self {
            item_type: AuctionType::Riven,
            weapon_url_name: weapon_url_name.to_string(),
            mod_name: Some(mod_name.to_string()),
            attributes: Some(attributes),
            re_rolls: Some(re_rolls),
            mastery_level: Some(mastery_level),
            mod_rank: Some(mod_rank),
            polarity: Some(polarity.to_string()),
            quirk: None,
            element: None,
            having_ephemera: None,
            damage: None,
        }
    }
    pub fn new_lich(
        weapon_url_name: &str,
        quirk: &str,
        element: &str,
        having_ephemera: bool,
        damage: i32,
    ) -> Self {
        Self {
            item_type: AuctionType::Lich,
            weapon_url_name: weapon_url_name.to_string(),
            mod_name: None,
            attributes: None,
            re_rolls: None,
            mastery_level: None,
            mod_rank: None,
            polarity: None,
            quirk: Some(quirk.to_string()),
            element: Some(element.to_string()),
            having_ephemera: Some(having_ephemera),
            damage: Some(damage),
        }
    }
    pub fn new_sister(
        weapon_url_name: &str,
        quirk: &str,
        element: &str,
        having_ephemera: bool,
        damage: i32,
    ) -> Self {
        let mut a =
            CreateAuctionItem::new_lich(weapon_url_name, quirk, element, having_ephemera, damage);
        a.item_type = AuctionType::Sister;
        a
    }
}
