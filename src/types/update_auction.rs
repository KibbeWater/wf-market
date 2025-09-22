use serde::Serialize;

#[derive(Serialize, Default)]
pub struct UpdateAuctionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyout_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimal_reputation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing)]
    pub properties: Option<serde_json::Value>, // Additional properties for the order
}

impl UpdateAuctionParams {
    pub fn new() -> Self {
        UpdateAuctionParams::default()
    }

    pub fn with_buyout_price(mut self, buyout_price: Option<u32>) -> Self {
        self.buyout_price = buyout_price;
        self
    }
    pub fn with_minimal_reputation(mut self, minimal_reputation: u32) -> Self {
        self.minimal_reputation = Some(minimal_reputation);
        self
    }
    pub fn with_note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
    pub fn with_starting_price(mut self, starting_price: u32) -> Self {
        self.starting_price = Some(starting_price);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
}
