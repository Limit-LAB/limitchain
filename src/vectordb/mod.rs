pub mod milvus;

use serde_json::Value;

#[async_trait::async_trait]
trait VectorDB {
    async fn query(&self, vector: Vec<f32>, limit: usize) -> anyhow::Result<Vec<(Value, f32)>>;
}