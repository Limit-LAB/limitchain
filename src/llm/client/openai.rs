use std::{collections::BTreeMap, sync::OnceLock};

use async_openai::{
    config::{Config, OPENAI_API_BASE, OPENAI_ORGANIZATION_HEADER},
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
};
use http::{header::AUTHORIZATION, HeaderMap};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::
    schema::{Generation, Message};

use crate::llm::LLM;

/// Configuration for OpenAI API
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_base: String,
    pub api_key: String,
    pub org_id: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_base: std::env::var("OPENAI_API_BASE")
                .unwrap_or_else(|_| OPENAI_API_BASE.to_string()),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string()),
            org_id: Default::default(),
        }
    }
}

impl Config for OpenAIConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if !self.org_id.is_empty() {
            headers.insert(
                OPENAI_ORGANIZATION_HEADER,
                self.org_id.as_str().parse().unwrap(),
            );
        }

        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.api_key).as_str().parse().unwrap(),
        );

        headers
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    fn api_base(&self) -> &str {
        &self.api_base
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn query(&self) -> Vec<(&str, &str)> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIClient {
    pub model: String,
    pub temperature: Option<f32>, // min: 0, max: 2, default: 1,
    pub top_p: Option<f32>,       // min: 0, max: 1, default: 1
    pub n: Option<u8>,            // min:1, max: 128, default: 1
    pub max_tokens: Option<u16>,
    pub presence_penalty: Option<f32>, // min: -2.0, max: 2.0, default 0
    pub frequency_penalty: Option<f32>, // min: -2.0, max: 2.0, default: 0
    pub user: Option<String>,

    #[serde(skip)]
    pub config: OpenAIConfig,
    #[serde(skip)]
    pub(crate) client: OnceLock<async_openai::Client<OpenAIConfig>>,
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self {
            config: Default::default(),
            model: "gpt-3.5-turbo".to_string(), // "gpt-3.5-turbo
            temperature: Default::default(),
            top_p: Default::default(),
            n: Default::default(),
            max_tokens: Default::default(),
            presence_penalty: Default::default(),
            frequency_penalty: Default::default(),
            user: Default::default(),
            client: Default::default(),
        }
    }
}

impl PartialEq for OpenAIClient {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Eq for OpenAIClient {}

// ====== LLM ======

fn message_to_chat_completion_request_message(message: Message) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage {
        role: match message.role.as_str().to_lowercase().as_str() {
            "user" => async_openai::types::Role::User,
            "assistant" => async_openai::types::Role::Assistant,
            "system" => async_openai::types::Role::System,
            _ => panic!("Invalid role"),
        },
        content: Some(message.content),
        ..Default::default()
    }
}

#[async_trait::async_trait]
impl LLM for OpenAIClient {
    fn name(&self) -> &'static str {
        "OpenAI"
    }

    async fn generate(&self, input: Vec<Message>, stop: Vec<String>) -> Generation {
        let client = self
            .client
            .get_or_init(|| async_openai::Client::with_config(self.config.clone()));
        let res = client
            .chat()
            .create(CreateChatCompletionRequest {
                model: self.model.clone(),
                messages: input
                    .into_iter()
                    .map(message_to_chat_completion_request_message)
                    .collect(),
                stop: Some(async_openai::types::Stop::StringArray(stop)),
                n: self.n,
                max_tokens: self.max_tokens,
                temperature: self.temperature,
                top_p: self.top_p,
                presence_penalty: self.presence_penalty,
                frequency_penalty: self.frequency_penalty,
                user: self.user.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        Generation {
            text: res
                .choices
                .iter()
                .map(|choice| Message {
                    role: choice.message.role.clone().to_string(),
                    content: choice.message.content.clone().unwrap(),
                })
                .collect(),
            info: {
                if let Some(usage) = res.usage {
                    serde_json::to_value(usage).ok()
                } else {
                    None
                }
            },
        }
    }
}

#[tokio::test]
async fn test_openai_client() {
    use std::str::FromStr;

    dotenvy::dotenv().unwrap();
    let client = OpenAIClient {
        n: Some(3),
        ..Default::default()
    };
    let res = client
        .generate(
            vec![
                Message::from_str("SYSTEM: you are lemon's AI cute maid Ashly, lemon is a programmer, you speak cutely and uses emoji alot, you never conclute things").unwrap(),
                Message::from_str("USER: write something about comparing rust and go").unwrap(),
            ],
            vec!["stop".to_string()],
        )
        .await;
    println!("{:#?}", res)
}
