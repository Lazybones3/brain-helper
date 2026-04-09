use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub instrument_type: String,
    pub region: String,
    pub universe: String,
    pub delay: i32,
    pub decay: i32,
    pub neutralization: String,
    pub truncation: f64,
    pub pasteurization: String,
    pub language: String,
    pub nan_handling: String,
    pub unit_handling: String,
    pub max_trade: String,
    pub visualization: bool,
    #[serde(default)]
    pub test_period: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_handling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_limit: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            instrument_type: "EQUITY".to_string(),
            region: "USA".to_string(),
            universe: "TOP3000".to_string(),
            delay: 1,
            decay: 0,
            neutralization: "SUBINDUSTRY".to_string(),
            truncation: 0.1,
            pasteurization: "ON".to_string(),
            test_period: "P0Y0M0D".to_string(),
            unit_handling: "VERIFY".to_string(),
            nan_handling: "OFF".to_string(),
            max_trade: "OFF".to_string(),
            language: "FASTEXPR".to_string(),
            visualization: false,
            selection_handling: None,
            selection_limit: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AlphaConfig {
    #[serde(rename = "REGULAR")]
    Regular { settings: Settings, regular: String },
    #[serde(rename = "SUPER")]
    Super {
        settings: Settings,
        combo: String,
        selection: String,
    },
}
