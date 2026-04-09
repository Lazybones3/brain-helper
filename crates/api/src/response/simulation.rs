use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationResult {
    pub alpha_id: String,
    pub regular: String,
}