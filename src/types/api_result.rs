use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct ApiResultV1<T> {
    pub payload: T,
}

#[derive(Clone, Deserialize)]
pub struct ApiResultV2<T> {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub data: T,
}
