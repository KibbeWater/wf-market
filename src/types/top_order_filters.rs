use serde::Serialize;

use crate::enums::StatusType;

#[derive(Clone, Default, Serialize)]
pub struct TopOrdersFilters {
    pub rank: Option<u32>,
    #[serde(rename = "rankLt")]
    pub rank_lt: Option<u32>,

    pub charges: Option<u32>,
    #[serde(rename = "chargesLt")]
    pub charges_lt: Option<u32>,

    #[serde(rename = "amberStars")]
    pub amber_stars: Option<u32>,
    #[serde(rename = "amberStarsLt")]
    pub amber_stars_lt: Option<u32>,

    #[serde(rename = "cyanStars")]
    pub cyan_stars: Option<u32>,
    #[serde(rename = "cyanStarsLt")]
    pub cyan_stars_lt: Option<u32>,

    pub subtype: Option<String>,

    #[serde(skip)]
    pub user_activity: Option<StatusType>,
}

impl TopOrdersFilters {
    pub fn new() -> Self {
        TopOrdersFilters::default()
    }

    pub fn with_rank(mut self, rank: u32) -> Self {
        self.rank = Some(rank);
        self
    }

    pub fn with_rank_lt(mut self, rank_lt: u32) -> Self {
        self.rank_lt = Some(rank_lt);
        self
    }

    pub fn with_charges(mut self, charges: u32) -> Self {
        self.charges = Some(charges);
        self
    }

    pub fn with_charges_lt(mut self, charges_lt: u32) -> Self {
        self.charges_lt = Some(charges_lt);
        self
    }

    pub fn with_amber_stars(mut self, amber_stars: u32) -> Self {
        self.amber_stars = Some(amber_stars);
        self
    }

    pub fn with_amber_stars_lt(mut self, amber_stars_lt: u32) -> Self {
        self.amber_stars_lt = Some(amber_stars_lt);
        self
    }

    pub fn with_cyan_stars(mut self, cyan_stars: u32) -> Self {
        self.cyan_stars = Some(cyan_stars);
        self
    }

    pub fn with_cyan_stars_lt(mut self, cyan_stars_lt: u32) -> Self {
        self.cyan_stars_lt = Some(cyan_stars_lt);
        self
    }

    pub fn with_subtype<S: Into<String>>(mut self, subtype: S) -> Self {
        self.subtype = Some(subtype.into());
        self
    }

    pub fn with_user_activity(mut self, user_activity: StatusType) -> Self {
        self.user_activity = Some(user_activity);
        self
    }
}
