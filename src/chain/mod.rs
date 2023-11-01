pub mod character_chain;
pub mod llm_chain;
pub mod map_reduce;
pub mod map_rerank;
pub mod seq_chain;

use std::{
    collections::{BTreeMap},
};

use serde::Serialize;


use crate::{
    prompt_template::PromptTemplate,
    schema::{Generation, Message, memory::Memory},
};

#[async_trait::async_trait]
pub trait LLM: Serialize + Send + Sync {
    fn name(&self) -> &'static str;
    async fn generate(&self, input: Vec<Message>, stop: Vec<String>) -> Generation;
    // async fn generate_stream(&self, input: Vec<Message>, stop: Vec<String>) -> Pin<Box<dyn Stream<Item = Generation> + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait Chain: Serialize {
    // ----- prepare -----
    fn get_input_keys(&self) -> Vec<String>;
    fn get_output_keys(&self) -> Vec<String>;
    fn get_prompt_template(&self) -> PromptTemplate;
    /// prepare_prompt function generates the prompt from the input
    fn prepare_prompt(&self, input: &BTreeMap<String, String>) -> Option<Message> {
        Some(Message {
            role: "user".to_string(),
            content: self.get_prompt_template().format(input)?,
        })
    }
    /// better override this
    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>>;

    // ----- execute -----
    /// generate function generates the output from the input and keeps in raw format
    async fn generate(
        &self,
        memory: Option<&Box<dyn Memory + Send + Sync>>,
        llm: &impl LLM,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<Generation> {
        let prompt = self.prepare_prompt(input);
        if let Some(mem) = memory {
            let mut his = mem.get_history().await.unwrap();
            his.push(prompt?);
            Some(llm.generate(his, stop).await)
        } else {
            let mut his = Vec::new();
            his.push(prompt?);
            Some(llm.generate(his, stop).await)
        }
    }
    /// apply function generates the output from the input
    async fn apply(
        &self,
        memory: Option<&Box<dyn Memory + Send + Sync>>,
        llm: &impl LLM,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<BTreeMap<String, Message>> {
        let generation = self.generate(memory, llm, input, stop).await;
        Some(self.create_output(generation?)?)
    }
    /// predict function generates the output from the input, default implementation is to call apply
    async fn predict(
        &self,
        memory: Option<&Box<dyn Memory + Send + Sync>>,
        llm: &impl LLM,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<BTreeMap<String, Message>> {
        self.apply(memory, llm, input, stop).await
    }
}
