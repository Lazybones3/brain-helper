use std::fs::File;
use std::io::BufWriter;
use serde_json::Value;

pub fn save_json(v: &Value, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, v)?;
    Ok(())
}