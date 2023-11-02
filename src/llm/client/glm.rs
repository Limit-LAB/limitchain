use serde::{Serialize, Deserialize};
use serde_json::json;
use zhipuai_sdk_rust::models::*;
use zhipuai_sdk_rust::models::characterglm::CharacterGLMMeta;
use crate::schema::{Message, Generation};
use crate::llm::{LLM, Embedding};
use zhipuai_sdk_rust::models::chatglm::ChatGLMInvokeParam;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMClient {
    invoke_param: serde_json::Value,
    pub model: String,
}

impl Default for GLMClient {
    fn default() -> Self {
        Self {
            invoke_param: json!(ChatGLMInvokeParam::default()),
            model: "chatglm_turbo".to_string(),
        }
    }
}

impl GLMClient {
    pub fn as_character(mut self, meta: CharacterGLMMeta) -> Self {
        self.invoke_param.as_object_mut().unwrap().insert("meta".to_string(), json!(meta));
        Self {
            model: "characterglm".to_string(),
            invoke_param: self.invoke_param,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMEmbeddingClient {
}

#[async_trait::async_trait]
impl Embedding for GLMEmbeddingClient {
    fn name(&self) -> &'static str {
        "ChatGLM Text Embedding"
    }
    async fn encode(&self, input: String) -> Vec<f32> {
        let model = Model::TextEmbedding;
        let res = model.invoke(InvokeMeta { prompt: json!(input), invoke_param: json!({}) }).await;
        res["data"]["embedding"].as_array().unwrap().into_iter().map(|x| x.as_f64().unwrap() as f32).collect()
    }
}

#[async_trait::async_trait]
impl LLM for GLMClient {
    fn name(&self) -> &'static str {
        "ChatGLM"
    }
    async fn generate(&self, input: Vec<Message>, _stop: Vec<String>) -> Generation {
        let invoke_prompt = json!(input);


        let model = match self.model.as_str() {
            "chatglm_130b" => Model::ChatGLM130b,
            "chatglm_6b" => Model::ChatGLM6b,
            "chatglm_turbo" => Model::ChatGLMTurbo,
            "characterglm" => Model::CharacterGLM,
            _ => panic!("unknown model")
        };

        let res = model.invoke(InvokeMeta { prompt:invoke_prompt, invoke_param: self.invoke_param.clone() }).await;

        Generation {
            text: res["data"]["choices"].as_array().unwrap().into_iter().map(|x| Message{role: x["role"].to_string(), content: x["content"].to_string()}).collect(),
            info: Some(
                res["data"]["usage"].clone()
            )
        }
    }
}

#[tokio::test]
async fn test_text_embedding() {
    dotenvy::dotenv().unwrap();
    let distance_fn = |r1: &Vec<f32>, r2: &Vec<f32>| {
        let mut sum = 0.0;
        for i in 0..r1.len() {
            sum += (r1[i] - r2[i]).powf(2.0);
        }
        sum.sqrt()
    };

    let r1 = GLMEmbeddingClient{}.encode("你好吗?".to_string()).await;
    let r2 = GLMEmbeddingClient{}.encode("how are you?".to_string()).await;
    // calculate distance between r1 r2 sentences
    let sum12 = distance_fn(&r1, &r2);
    println!("distance r1 r2: {}", sum12);

    let r3 = GLMEmbeddingClient{}.encode("今天是个艳阳天".to_string()).await;
    // calculate distance between r1 r3 sentences
    let sum13 = distance_fn(&r1, &r3);
    println!("distance r1 r3: {}", sum13);

    assert!(sum12 < sum13)
}