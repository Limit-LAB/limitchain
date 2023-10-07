pub mod docx;
pub mod markdown;
pub mod xml;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub text: String,
    pub meta: serde_json::Value,
}

pub trait DocumentLoader {
    fn load_mem(&self, mem: &str) -> anyhow::Result<Vec<Document>>;
    fn load_file(&self, path: &str) -> anyhow::Result<Vec<Document>>;
}
