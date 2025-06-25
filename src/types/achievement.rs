use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "achievedAt")]
    pub achieved_at: Option<String>,
}