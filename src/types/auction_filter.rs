use serde::Serialize;

use crate::enums::{AuctionType, Polarity, StatusType};

#[derive(Clone, Default, Serialize)]
pub struct AuctionFilter {
    #[serde(rename = "type")]
    pub auction_type: AuctionType,
    pub weapon_url_name: String,
    pub positive_stats: Option<String>,
    pub negative_stats: Option<String>,
    pub sort_by: Option<String>,
    pub polarity: Option<Polarity>,
    pub mastery_rank_min: Option<u32>,
    pub mastery_rank_max: Option<u32>,
    pub re_rolls_min: Option<u32>,
    pub re_rolls_max: Option<u32>,
    pub buyout_policy: Option<String>,
    pub mod_rank: Option<String>,

    #[serde(skip)]
    pub user_activity: Option<StatusType>,
}

impl AuctionFilter {
    pub fn new(auction_type: AuctionType, weapon_url_name: &str) -> Self {
        AuctionFilter {
            auction_type: auction_type,
            positive_stats: None,
            negative_stats: None,
            sort_by: None,
            weapon_url_name: weapon_url_name.to_string(),
            polarity: None,
            mastery_rank_min: None,
            mastery_rank_max: None,
            re_rolls_min: None,
            re_rolls_max: None,
            buyout_policy: None,
            mod_rank: None,
            user_activity: None,
        }
    }

    pub fn with_positive_stats<S: Into<String>>(mut self, positive_stats: S) -> Self {
        self.positive_stats = Some(positive_stats.into());
        self
    }

    pub fn with_negative_stats<S: Into<String>>(mut self, negative_stats: S) -> Self {
        self.negative_stats = Some(negative_stats.into());
        self
    }

    pub fn with_sort_by<S: Into<String>>(mut self, sort_by: S) -> Self {
        self.sort_by = Some(sort_by.into());
        self
    }

    pub fn with_polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = Some(polarity);
        self
    }

    pub fn with_mastery_rank_min(mut self, mastery_rank_min: u32) -> Self {
        self.mastery_rank_min = Some(mastery_rank_min);
        self
    }

    pub fn with_mastery_rank_max(mut self, mastery_rank_max: u32) -> Self {
        self.mastery_rank_max = Some(mastery_rank_max);
        self
    }

    pub fn with_re_rolls_min(mut self, re_rolls_min: u32) -> Self {
        self.re_rolls_min = Some(re_rolls_min);
        self
    }

    pub fn with_re_rolls_max(mut self, re_rolls_max: u32) -> Self {
        self.re_rolls_max = Some(re_rolls_max);
        self
    }

    pub fn with_buyout_policy<S: Into<String>>(mut self, buyout_policy: S) -> Self {
        self.buyout_policy = Some(buyout_policy.into());
        self
    }

    pub fn with_mod_rank<S: Into<String>>(mut self, mod_rank: S) -> Self {
        self.mod_rank = Some(mod_rank.into());
        self
    }

    pub fn with_user_activity(mut self, user_activity: StatusType) -> Self {
        self.user_activity = Some(user_activity);
        self
    }
}
