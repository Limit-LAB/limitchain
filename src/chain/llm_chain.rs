use std::collections::BTreeMap;

use async_trait::async_trait;
use itertools::Itertools;
use serde::Serialize;

use crate::{
    prompt_template::PromptTemplate,
    schema::{Generation, Message},
};

use super::{Chain, LLM};

/// {question} -> {answer}
#[derive(Debug, Clone, Serialize)]
pub struct LLMChain<Executor: Send + Sync + Serialize + LLM + Clone> {
    prompt_template: Option<PromptTemplate>,
    executor: Executor,
}

impl<Executor: Send + Sync + Serialize + LLM + Clone> LLMChain<Executor> {
    pub fn new(prompt_template: Option<PromptTemplate>, executor: Executor) -> Self {
        Self {
            prompt_template,
            executor,
        }
    }
}

#[async_trait]
impl<Executor: Send + Sync + Serialize + LLM + Clone> Chain for LLMChain<Executor> {
    fn get_input_keys(&self) -> Vec<String> {
        self.prompt_template
            .as_ref()
            .map_or(vec!["question".to_string()], |t| {
                t.variables.keys().cloned().collect_vec()
            })
    }

    fn get_output_keys(&self) -> Vec<String> {
        vec!["answer".to_string()]
    }

    fn get_prompt_template(&self) -> PromptTemplate {
        self.prompt_template
            .clone()
            .unwrap_or_else(|| PromptTemplate::from("{question}".to_string()))
    }

    fn get_llm(&self) -> impl LLM {
        self.executor.clone()
    }

    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>> {
        let mut output = BTreeMap::new();
        output.insert("answer".to_string(), generation.text[0].clone());
        Some(output)
    }
}

#[tokio::test]
async fn test_llm_chain_openai() {
    use crate::btreemap;
    use crate::client::openai::*;
    dotenvy::dotenv().unwrap();

    let chain = LLMChain {
        prompt_template: Some(PromptTemplate::from("{question}".to_string())),
        executor: OpenAIClient::default(),
    };

    let res = chain
        .apply(
            &btreemap! {
                "question".to_string() => "What is human?".to_string()
            },
            vec!["stop".to_string()],
        )
        .await;

    println!("{:?}", res);
}
