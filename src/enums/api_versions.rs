pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "https://api.warframe.market/v1",
            ApiVersion::V2 => "https://api.warframe.market/v2",
        }
    }
}
