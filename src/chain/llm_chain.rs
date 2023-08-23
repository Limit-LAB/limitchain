use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest};

use crate::client::openai::OpenAIClient;

use super::*;

#[async_trait::async_trait]
impl LLM for OpenAIClient {
    async fn generate(&self, input: Vec<String>, stop: Vec<String>) -> Generation {
        let client = self
            .client
            .get_or_init(|| async_openai::Client::with_config(self.config.clone()));
        let res = client
            .chat()
            .create(CreateChatCompletionRequest {
                model: "gpt-3.5-turbo".to_string(),
                messages: vec![
                    ChatCompletionRequestMessage {
                        role: async_openai::types::Role::System,
                        content: Some("you are personal assistant".to_string()),
                        ..Default::default()
                    },
                    ChatCompletionRequestMessage {
                        role: async_openai::types::Role::User,
                        content: Some(input[0].clone()),
                        ..Default::default()
                    },
                ],
                stop: Some(async_openai::types::Stop::StringArray(stop)),
                ..Default::default()
            })
            .await
            .unwrap();

        println!("{:?}", res);
        todo!()
    }
}

#[tokio::test]
async fn test_openai_client() {
    let client = OpenAIClient {
        ..Default::default()
    };
    let _res = client
        .generate(vec!["who are you?".to_string()], vec!["stop".to_string()])
        .await;
}
