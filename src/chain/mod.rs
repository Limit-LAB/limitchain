pub mod llm_chain;

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{prompt_template::PromptTemplate, schema::Generation};

#[async_trait::async_trait]
pub trait LLM: Serialize + Send + Sync {
    async fn generate(&self, input: Vec<String>, stop: Vec<String>) -> Generation;
}

#[async_trait::async_trait]
pub trait Chain: Serialize {
    fn get_input_keys(&self) -> Vec<String>;
    fn get_output_keys(&self) -> Vec<String>;
    fn get_prompt_template(&self) -> PromptTemplate;
    fn get_llm(&self) -> impl LLM;

    /// prepare_prompt function generates the prompt from the input
    fn prepare_prompt(&self, input: &BTreeMap<String, String>) -> Option<String> {
        self.get_prompt_template().format(input)
    }
    fn prepare_prompt_batch(&self, inputs: &Vec<BTreeMap<String, String>>) -> Vec<Option<String>> {
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
    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, String>> {
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
    ) -> Option<BTreeMap<String, String>> {
        let generation = self.generate(input, stop).await;
        Some(self.create_output(generation?)?)
    }
    async fn apply_batch(
        &self,
        inputs: &Vec<BTreeMap<String, String>>,
        stop: Vec<String>,
    ) -> Vec<Option<BTreeMap<String, String>>> {
        let futs = inputs.iter().map(|input| self.apply(input, stop.clone()));
        futures::future::join_all(futs).await
    }

    /// predict function generates the output from the input, default implementation is to call apply
    async fn predict(
        &self,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<BTreeMap<String, String>> {
        self.apply(input, stop).await
    }
    async fn predict_batch(
        &self,
        inputs: &Vec<BTreeMap<String, String>>,
        stop: Vec<String>,
    ) -> Vec<Option<BTreeMap<String, String>>> {
        self.apply_batch(inputs, stop).await
    }
}
