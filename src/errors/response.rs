use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseError {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub data: Option<serde_json::Value>,
    pub error: super::ApiErrorBody,
}
impl ResponseError {
    pub fn new(
        api_version: &str,
        data: Option<serde_json::Value>,
        error: super::ApiErrorBody,
    ) -> Self {
        ResponseError {
            api_version: api_version.to_string(),
            data,
            error,
        }
    }
    pub fn from_v1(v_body: Value) -> Self {
        let error: Value = v_body["error"].clone();
        let mut error_map = HashMap::new();

        if error.is_string() {
            error_map.insert("error".to_string(), error.as_str().unwrap().to_string());
        } else if error.is_object() {
            match serde_json::from_value::<HashMap<String, Value>>(error) {
                Ok(errors) => {
                    for (key, value) in errors {
                        Self::collect_error_values(&key, &value, &mut error_map);
                    }
                }
                Err(e) => {
                    error_map.insert(
                        "parse_error".to_string(),
                        format!("Could not parse error messages: {}", e),
                    );
                }
            }
        }

        ResponseError {
            api_version: "0.0.1".to_string(),
            data: None,
            error: super::ApiErrorBody {
                request: None,
                inputs: Some(error_map),
            },
        }
    }

    fn collect_error_values(key: &str, value: &Value, error_map: &mut HashMap<String, String>) {
        if value.is_array() {
            match serde_json::from_value::<Vec<String>>(value.clone()) {
                Ok(messages) => {
                    error_map.insert(key.to_string(), messages.join(", "));
                }
                Err(_) => {
                    // Try to handle array of objects
                    if let Some(array) = value.as_array() {
                        for (i, item) in array.iter().enumerate() {
                            Self::collect_error_values(&format!("{}[{}]", key, i), item, error_map);
                        }
                    }
                }
            }
        } else if value.is_object() {
            // Handle nested objects like {"attributes":{"0":{"value":["app.form.invalid"]}}}
            if let Some(obj) = value.as_object() {
                for (nested_key, nested_value) in obj {
                    Self::collect_error_values(
                        &format!("{}.{}", key, nested_key),
                        nested_value,
                        error_map,
                    );
                }
            }
        } else if value.is_string() {
            error_map.insert(key.to_string(), value.as_str().unwrap().to_string());
        } else {
            error_map.insert(key.to_string(), format!("{:?}", value));
        }
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Start with API version
        write!(f, "API v{}", self.api_version)?;

        // Add error details from the error body
        if let Some(ref inputs) = self.error.inputs {
            if !inputs.is_empty() {
                write!(f, " - ")?;
                let error_messages: Vec<String> = inputs
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect();
                write!(f, "{}", error_messages.join(", "))?;
            }
        }

        // Add request info if available
        if let Some(ref request) = self.error.request {
            write!(f, " (request: {:?})", request)?;
        }

        Ok(())
    }
}
