use serde::{Deserialize, Serialize};

pub mod auth;
pub mod dataset;
pub mod datafield;
pub mod operator;
pub mod simulation;

#[derive(Debug, Deserialize)]
pub struct ResultsResponse<T> {
    pub count: usize,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdNamePair {
    pub id: String,
    pub name: String,
}
