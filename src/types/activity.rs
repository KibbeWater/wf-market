use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Activity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub description: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<String>,
}