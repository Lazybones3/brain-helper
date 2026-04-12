use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub category: String,
    pub definition: String,
    pub description: String,
    // handle documentation: null
    pub documentation: Option<String>,
    pub level: Option<String>,
    pub name: String,
    pub scope: Vec<String>,
}
