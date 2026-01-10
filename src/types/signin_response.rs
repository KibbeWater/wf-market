use serde::Deserialize;

// TODO: Use for Api version 1
#[derive(Deserialize)]
pub struct SigninResponse {
    pub avatar: Option<String>,
    pub linked_accounts: SigninLinkedAccounts,
    pub role: String,
    pub locale: String,
    pub background: Option<String>,
    pub crossplay: bool,
    pub platform: String,
    pub reputation: f64,
    pub has_mail: bool,
    pub region: String,
    pub written_reviews: i64,
    pub id: String,
    pub ingame_name: String,
    pub slug: String,
    pub unread_messages: i64,
    pub banned: bool,
    pub check_code: String,
    pub verification: bool,
    pub anonymous: bool,
}

// TODO: Use for Api version 1
#[derive(Deserialize)]
pub struct SigninLinkedAccounts {
    pub steam_profile: bool,
    pub patreon_profile: bool,
    pub xbox_profile: bool,
    pub discord_profile: bool,
    pub github_profile: bool,
}
