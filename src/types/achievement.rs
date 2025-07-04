use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct Achievement {
    pub id: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub achievement_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<bool>,
    #[serde(rename = "reputationBonus", skip_serializing_if = "Option::is_none")]
    pub reputation_bonus: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<i32>,
    pub i18n: HashMap<String, AchievementTranslation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AchievementState>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AchievementTranslation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AchievementState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<i32>,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}
