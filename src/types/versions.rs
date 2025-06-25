use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionsResponse {
    pub apps: AppVersions,
    pub collections: Collections,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppVersions {
    pub ios: String,
    pub android: String,
    #[serde(rename = "minIos")]
    pub min_ios: String,
    #[serde(rename = "minAndroid")]
    pub min_android: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Collections {
    pub items: String,
    pub rivens: String,
    pub liches: String,
    pub sisters: String,
    pub missions: String,
    pub npcs: String,
    pub locations: String,
}
