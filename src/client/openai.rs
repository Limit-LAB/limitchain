use std::sync::OnceLock;

use async_openai::config::{Config, OPENAI_API_BASE, OPENAI_ORGANIZATION_HEADER};
use http::{header::AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};

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
    pub config: OpenAIConfig,
    pub model: String,
    pub temperature: Option<f32>, // min: 0, max: 2, default: 1,
    pub top_p: Option<f32>,       // min: 0, max: 1, default: 1
    pub n: Option<u8>,            // min:1, max: 128, default: 1
    pub max_tokens: Option<u16>,
    pub presence_penalty: Option<f32>, // min: -2.0, max: 2.0, default 0
    pub frequency_penalty: Option<f32>, // min: -2.0, max: 2.0, default: 0
    pub user: Option<String>,

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
