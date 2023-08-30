pub mod llm_chain;

use std::{collections::BTreeMap, str::FromStr};

use serde::Serialize;

use crate::{prompt_template::PromptTemplate, schema::Generation};

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
impl FromStr for Message {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split ':' and take the first one as role
        let mut split = s.splitn(2, ": ");
        if let Some(role) = split.next() {
            if let Some(content) = split.next() {
                return Ok(Message {
                    role: role.to_string(),
                    content: content.to_string(),
                });
            } else {
                return Err("Invalid message".to_string());
            }
        } else {
            return Err("Invalid message".to_string());
        }
    }
}

#[async_trait::async_trait]
pub trait LLM: Serialize + Send + Sync {
    async fn generate(&self, input: Vec<Message>, stop: Vec<String>) -> Generation;
}

#[async_trait::async_trait]
pub trait Chain: Serialize {
    fn get_input_keys(&self) -> Vec<String>;
    fn get_output_keys(&self) -> Vec<String>;
    fn get_prompt_template(&self) -> PromptTemplate;
    fn get_llm(&self) -> impl LLM;

    /// prepare_prompt function generates the prompt from the input
    fn prepare_prompt(&self, input: &BTreeMap<String, String>) -> Option<Message> {
        Some(Message {
            role: "user".to_string(),
            content: self.get_prompt_template().format(input)?,
        })
    }
    fn prepare_prompt_batch(&self, inputs: &Vec<BTreeMap<String, String>>) -> Vec<Option<Message>> {
        inputs
            .iter()
            .map(|input| self.prepare_prompt(input))
            .collect()
    }

    /// generate function generates the output from the input and keeps in raw format
    async fn generate(
        &self,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<Generation> {
        let prompt = self.prepare_prompt(input);
        let llm = self.get_llm();
        Some(llm.generate(vec![prompt?], stop).await)
    }
    async fn generate_batch(
        &self,
        inputs: &Vec<BTreeMap<String, String>>,
        stop: Vec<String>,
    ) -> Vec<Option<Generation>> {
        let prompts = self.prepare_prompt_batch(inputs);
        let llm = self.get_llm();
        let mut result = Vec::new();
        for prompt in prompts {
            if let Some(prompt) = prompt {
                result.push(Some(llm.generate(vec![prompt], stop.clone()).await));
            } else {
                result.push(None);
            }
        }
        result
    }

    /// better override this
    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>> {
        // take the first generation
        let text = generation.text.first()?;
        let mut output = BTreeMap::new();
        for key in self.get_output_keys() {
            output.insert(key, text.clone());
        }
        Some(output)
    }

    /// apply function generates the output from the input
    async fn apply(
        &self,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<BTreeMap<String, Message>> {
        let generation = self.generate(input, stop).await;
        Some(self.create_output(generation?)?)
    }
    async fn apply_batch(
        &self,
        inputs: &Vec<BTreeMap<String, String>>,
        stop: Vec<String>,
    ) -> Vec<Option<BTreeMap<String, Message>>> {
        let futs = inputs.iter().map(|input| self.apply(input, stop.clone()));
        futures::future::join_all(futs).await
    }

    /// predict function generates the output from the input, default implementation is to call apply
    async fn predict(
        &self,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<BTreeMap<String, Message>> {
        self.apply(input, stop).await
    }
    async fn predict_batch(
        &self,
        inputs: &Vec<BTreeMap<String, String>>,
        stop: Vec<String>,
    ) -> Vec<Option<BTreeMap<String, Message>>> {
        self.apply_batch(inputs, stop).await
    }
}
