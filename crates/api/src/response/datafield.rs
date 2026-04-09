use serde::Deserialize;

use crate::response::IdNamePair;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataField {
    pub id: String,
    pub description: String,
    pub alpha_count: i32,
    pub user_count: i32,
    pub delay: i32,
    pub region: String,
    pub universe: String,
    pub coverage: f64,
    #[serde(default)]
    pub date_coverage: f64,
    pub pyramid_multiplier: f64,
    
    // Nested objects
    pub dataset: IdNamePair,
    pub category: IdNamePair,
    pub subcategory: IdNamePair,
    
    // Enums/Collections
    pub themes: Vec<String>,
    
    #[serde(rename = "type")]
    pub field_type: String,
}
