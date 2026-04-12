use brain_api::Settings;
use brain_common::logger;
use brain_database::database;

use crate::app::BrainApp;

mod app;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();

    // database::init_table().await;
    let settings = Settings::default();
    let app = BrainApp::new(settings).await?;
    app.simulation("Image-Based Financial Prediction Dataset", Some("MATRIX".to_string()), Some("ts_delta(ts_delta(<field>, 5), 5)".to_string())).await?;
    Ok(())
}