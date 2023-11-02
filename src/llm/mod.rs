use serde::Serialize;

use crate::schema::{Message, Generation};

pub mod client;


#[async_trait::async_trait]
pub trait LLM: Serialize + Send + Sync {
    fn name(&self) -> &'static str;
    async fn generate(&self, input: Vec<Message>, stop: Vec<String>) -> Generation;
    // async fn generate_stream(&self, input: Vec<Message>, stop: Vec<String>) -> Pin<Box<dyn Stream<Item = Generation> + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait Embedding: Serialize + Send + Sync {
    fn name(&self) -> &'static str;
    async fn encode(&self, input: String) -> Vec<f32>;
}

