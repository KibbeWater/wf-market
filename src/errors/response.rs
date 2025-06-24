use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ResponseError {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub data: Option<serde_json::Value>,
    pub error: super::ApiErrorBody,
}
