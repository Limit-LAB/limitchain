use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest};
use itertools::Itertools;

use crate::client::openai::{OpenAIClient};

use super::*;

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
                let mut info = BTreeMap::new();
                // usage
                if let Some(usage) = res.usage {
                    info.insert("prompt_tokens".to_string(), usage.prompt_tokens.to_string());
                    info.insert(
                        "completion_tokens".to_string(),
                        usage.completion_tokens.to_string(),
                    );
                    info.insert("total_tokens".to_string(), usage.total_tokens.to_string());
                }
                // finish reason

                info.insert(
                    "finish_reason".to_string(),
                    res.choices
                        .iter()
                        .map(|c| c.finish_reason.clone())
                        .filter(|r| r.is_some())
                        .map(|r| r.unwrap())
                        .collect_vec()
                        .join(", "),
                );
                Some(info)
            },
        }
    }
}

#[tokio::test]
async fn test_openai_client() {
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
