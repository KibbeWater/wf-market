use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platinum: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_trade: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

impl UpdateOrderParams {
    pub fn new() -> Self {
        UpdateOrderParams::default()
    }

    pub fn with_platinum(mut self, platinum: u32) -> Self {
        self.platinum = Some(platinum);
        self
    }

    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn with_per_trade(mut self, per_trade: u32) -> Self {
        self.per_trade = Some(per_trade);
        self
    }

    pub fn with_rank(mut self, rank: u32) -> Self {
        self.rank = Some(rank);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }
}
