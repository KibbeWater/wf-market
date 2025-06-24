use serde::Deserialize;

#[derive(Clone, Deserialize, Eq, PartialEq, Debug)]
pub enum StatusType {
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "in_game", alias = "ingame")]
    InGame,
}
impl Default for StatusType {
    fn default() -> Self {
        StatusType::Offline
    }
}
