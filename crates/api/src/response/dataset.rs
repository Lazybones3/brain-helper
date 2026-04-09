use serde::Deserialize;

use crate::response::IdNamePair;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub alpha_count: i32,
    pub category: IdNamePair,
    pub subcategory: IdNamePair,
    pub coverage: f64,
    pub date_coverage: f64,
    pub delay: i32,
    pub description: String,
    pub field_count: i32,
    pub pyramid_multiplier: f64,
    pub region: String,
    pub research_papers: Vec<ResearchPaper>,
    pub themes: Vec<String>,
    pub universe: String,
    pub user_count: i32,
    pub value_score: f64,
}

#[derive(Debug, Deserialize)]
pub struct ResearchPaper {
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    pub paper_type: String,
}
