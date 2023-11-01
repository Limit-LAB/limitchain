use std::sync::OnceLock;
use serde_json::{Value, json};

use super::VectorDB;

pub struct MilvusClient {
    collection_name: String,
    token: String,
    host: String,
    filter: Option<String>,
    offset: Option<usize>,
    output_fields: Vec<String>,

    client: OnceLock<reqwest::Client>
}

impl MilvusClient {
    pub fn new(collection_name: String, token: String, host: String, output_fields: Vec<String>) -> Self {
        Self {
            collection_name,
            token,
            host,
            output_fields,
            filter: None,
            offset: None,
            client: OnceLock::new(),
        }
    }
}

#[async_trait::async_trait]
impl VectorDB for MilvusClient {
    async fn query(&self, vector: Vec<f32>, limit: usize) -> anyhow::Result<Vec<(Value, f32)>> {
        let client = self.client.get_or_init(||{
            reqwest::Client::new()
        });
        let mut value = json!({
            "collectionName": self.collection_name,
            "outputFields": self.output_fields,
            "vector": vector,
            "limit": limit,
        });
        if let Some(filter) = &self.filter {
            value.as_object_mut().unwrap().insert("filter".to_string(), filter.clone().into());
        }
        if let Some(offset) = &self.offset {
            value.as_object_mut().unwrap().insert("offset".to_string(), offset.clone().into());
        }

        let res = client.post(self.host.clone() + "/v1/vector/search")
        .bearer_auth(self.token.clone())
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .json(&value)
        .send()
        .await?;

        println!("{:#?}", res.json().await?);

        todo!("")
    }
}

#[tokio::test]
async fn test() {
    
}