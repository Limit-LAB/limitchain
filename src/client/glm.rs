
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use hmac::{digest::KeyInit, Hmac};
use jwt::SigningAlgorithm;
use jwt::{ToBase64};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;


use crate::chain::LLM;
use crate::parser::unescape;
use crate::schema::{Generation, Message};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GLMConfig {
    pub api_key: String,
}

impl Default for GLMConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ZHIPU_API_KEY").unwrap_or_else(|_| "".to_string()),
        }
    }
}

fn create_jwt_token(api_key: &str, expire: std::time::Duration) -> Result<String> {
    let sp = api_key.split('.').collect::<Vec<_>>();
    let api_key = *sp.first().ok_or_else(|| anyhow!("Invalid API key"))?;
    let secret = *sp.last().ok_or_else(|| anyhow!("Invalid API key"))?;

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let exp = now + expire;

    let now_ts = now.as_millis();
    let exp_ts = exp.as_millis();

    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes())?;
    let header = json!(
        {"alg":"HS256","sign_type":"SIGN","typ":"JWT"}
    );

    let claims = json!({
        "api_key" : api_key,
        "exp" : exp_ts,
        "timestamp":now_ts
    });
    let header = header.to_base64()?;
    let claims = claims.to_base64()?;
    let signature = key.sign(&header, &claims)?;

    let token_string = [&*header, &*claims, &signature].join(".");
    Ok(token_string)
}

#[tokio::test]
async fn test_jwt() {
    dotenvy::dotenv().unwrap();

    let body = json!({
        "prompt": [
            {
                "role": "user",
                "content": "我问丁真你是哪个省的，为什么丁真回答 “我是妈妈生的？” 请给出我200字以上的答案。"
            }
        ],
    });

    let glm_config = GLMConfig::default();

    let token =
        create_jwt_token(&glm_config.api_key, std::time::Duration::from_secs(10000)).unwrap();

    let client = reqwest::Client::new();

    let resp = client
        .post("https://open.bigmodel.cn/api/paas/v3/model-api/chatglm_std/invoke")
        .header("Authorization", token)
        // header send json
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap();
    println!("{:#?}", resp.text().await);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMCharacterMeta {
    pub user_info: String,
    pub bot_info: String,
    pub bot_name: String,
    pub user_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMClient {
    pub model: String,
    pub meta: Option<GLMCharacterMeta>,
    pub temperature: Option<f32>,     // min: 0, max: 2, default: 1,
    pub top_p: Option<f32>,           // min: 0, max: 1, default: 1
    pub enable_refers: Option<bool>,  // enable ref search information on internet
    pub refers_query: Option<String>, // what to search

    #[serde(skip)]
    pub config: GLMConfig,
    #[serde(skip)]
    pub reqwest_client: OnceLock<reqwest::Client>,
}

impl Default for GLMClient {
    fn default() -> Self {
        Self {
            config: Default::default(),
            model: "chatglm_std".to_string(),
            meta: None,
            temperature: Default::default(),
            top_p: Default::default(),
            enable_refers: Default::default(),
            refers_query: Default::default(),
            reqwest_client: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl LLM for GLMClient {
    fn name(&self) -> &'static str {
        "GLM"
    }
    async fn generate(&self, input: Vec<Message>, _stop: Vec<String>) -> Generation {
        let mut input_json = json!({
            "prompt": input.iter().map(|x| {
                json!({
                    "role": x.role,
                    "content": x.content,
                })
            }).collect::<Vec<_>>(),

        });

        if self.meta.is_some() && self.model == "characterglm" {
            input_json.as_object_mut().unwrap().insert(
                "meta".to_string(),
                json!({
                    "user_info": self.meta.as_ref().unwrap().user_info,
                    "bot_info": self.meta.as_ref().unwrap().bot_info,
                    "bot_name": self.meta.as_ref().unwrap().bot_name,
                    "user_name": self.meta.as_ref().unwrap().user_name
                }),
            );
        }
        // temperature
        if self.temperature.is_some() {
            input_json
                .as_object_mut()
                .unwrap()
                .insert("temperature".to_string(), json!(self.temperature.unwrap()));
        }
        // top_p
        if self.top_p.is_some() {
            input_json
                .as_object_mut()
                .unwrap()
                .insert("top_p".to_string(), json!(self.top_p.unwrap()));
        }
        // enable_refers
        if self.enable_refers.is_some() {
            input_json.as_object_mut().unwrap().insert(
                "ref".to_string(),
                json!({
                    "enable": self.enable_refers.unwrap(),
                    "query": self.refers_query.as_ref().unwrap_or(&"".to_string()),
                }),
            );
        }

        let token =
            create_jwt_token(&self.config.api_key, std::time::Duration::from_secs(100)).unwrap();

        if !_stop.is_empty() {
            println!("GLMClient::generate: stop is not empty, but glm does not support stop yet");
        }

        let client = self.reqwest_client.get_or_init(|| reqwest::Client::new());

        let resp = client
            .post(format!(
                "https://open.bigmodel.cn/api/paas/v3/model-api/{}/invoke",
                self.model
            ))
            .header("Authorization", token)
            .header("Content-Type", "application/json")
            .body(input_json.to_string());

        let resp = resp.send().await.unwrap().json::<Response>().await.unwrap();

        Generation {
            text: resp
                .data
                .as_ref()
                .unwrap()
                .choices
                .iter()
                .map(|choice| Message {
                    role: choice.role.clone(),
                    content: unescape(&choice.content).unwrap(),
                })
                .collect(),
            info: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Choices {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Data {
    request_id: String,
    task_id: String,
    task_status: String,
    choices: Vec<Choices>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct Response {
    code: u32,
    msg: String,
    data: Option<Data>,
    usage: Option<Usage>,
    success: bool,
}
