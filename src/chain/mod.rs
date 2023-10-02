pub mod llm_chain;
pub mod map_reduce;
pub mod map_rerank;
pub mod seq_chain;

use std::{
    collections::{BTreeMap, VecDeque},
    str::FromStr,
};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    prompt_template::PromptTemplate,
    schema::{Generation, Message},
};

#[async_trait::async_trait]
pub trait LLM: Serialize + Send + Sync {
    async fn generate(&self, input: Vec<Message>, stop: Vec<String>) -> Generation;
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

#[async_trait::async_trait]
pub trait Memory {
    async fn push_front(&self, message: Message) -> anyhow::Result<()>;
    async fn push_back(&self, message: Message) -> anyhow::Result<()>;
    async fn pop_front(&self) -> anyhow::Result<Message>;
    async fn pop_back(&self) -> anyhow::Result<Message>;
    async fn get_history(&self) -> anyhow::Result<Vec<Message>>;
}

#[derive(Debug)]
pub struct InMemMemory {
    history: Mutex<VecDeque<Message>>,
}

#[async_trait::async_trait]
impl Memory for InMemMemory {
    async fn push_front(&self, message: Message) -> anyhow::Result<()> {
        self.history.lock().await.push_front(message);
        Ok(())
    }

    async fn push_back(&self, message: Message) -> anyhow::Result<()> {
        self.history.lock().await.push_back(message);
        Ok(())
    }

    async fn pop_front(&self) -> anyhow::Result<Message> {
        self.history
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("Memory is empty"))
    }

    async fn pop_back(&self) -> anyhow::Result<Message> {
        self.history
            .lock()
            .await
            .pop_back()
            .ok_or_else(|| anyhow::anyhow!("Memory is empty"))
    }

    async fn get_history(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self.history.lock().await.clone().into())
    }
}

impl From<Vec<Message>> for InMemMemory {
    fn from(history: Vec<Message>) -> Self {
        Self {
            history: Mutex::new(VecDeque::from(history)),
        }
    }
}
