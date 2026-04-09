use jiff::civil::DateTime;
use rust_decimal::Decimal;

#[derive(Debug, toasty::Model)]
#[table = "alpha_entity"]
pub struct AlphaEntity {
    #[key]
    pub id: String,
    pub crete_time: DateTime,
    #[default("".to_string())]
    pub regular: String,
    pub instrument_type: String,
    pub region: String,
    pub universe: String,
    pub delay: i32,
    pub decay: i32,
    pub neutralization: String,
    pub truncation: Decimal,
    pub pasteurization: String,
    pub test_period: String,
    pub unit_handling: String,
    pub nan_handling: String,
    pub max_trade: String,
    pub language: String,
    pub visualization: bool,
    pub selection_handling: String,
    pub selection_limit: i32,
    #[default("".to_string())]
    pub combo: String,
    #[default("".to_string())]
    pub selection: String,
}