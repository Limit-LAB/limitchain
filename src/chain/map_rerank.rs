use std::collections::BTreeMap;

use crate::{
    btreemap,
    chain::Chain,
    client::glm::GLMClient,
    prompt_template::PromptTemplate,
    schema::{Generation, Message},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{Memory, LLM};

/// usage: {1: {q1}, 2: {q2}} -> {output}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "chain_type")]
pub struct MapRerankChain<MapChain: Chain + Serialize + Send + Sync> {
    prompt_template: Option<PromptTemplate>,
    map_chain: MapChain,
}

#[async_trait::async_trait]
impl<MapChain: Chain + Serialize + Send + Sync> Chain for MapRerankChain<MapChain> {
    fn get_input_keys(&self) -> Vec<String> {
        let mut input_keys = self.map_chain.get_input_keys();
        input_keys.push("question".to_string());
        input_keys
    }

    fn get_output_keys(&self) -> Vec<String> {
        vec!["answer".to_string(), "score".to_string()]
    }

    fn get_prompt_template(&self) -> PromptTemplate {
        self.prompt_template.clone().unwrap_or_else(|| {
            PromptTemplate::from(
                "score the relativeness of the document to the answer from 0.0 to 1.0
Question: {question}
Doc: {answer}

output format: \\{\"score\": your score goes here, \"doc\": copy the Doc above \\}
for example: \\{\"score\": 0.5, \"doc\": \"blablabla\"\\}"
                    .to_string(),
            )
        })
    }

    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>> {
        let mut output = BTreeMap::new();
        output.insert("answer".to_string(), generation.text[0].clone());
        Some(output)
    }

    async fn generate(
        &self,
        memory: Option<&Box<dyn Memory + Send + Sync>>,
        llm: &impl LLM,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<Generation> {
        let mut inputs: Vec<BTreeMap<String, String>> = vec![];
        let mut input2 = input.clone();
        for (k, v) in input {
            if k.parse::<usize>().is_ok() {
                if let Ok(v) = serde_json::from_str(&v) {
                    inputs.push(v);
                    input2.remove(k);
                } else {
                    println!("Warning: MapReduceChain: value {} is not a json", v);
                }
            }
        }
        let input = input2;

        let futs = inputs.iter().map(|input| async {
            let mut his = Vec::new();
            let prompt = self.map_chain.prepare_prompt(input).unwrap();
            his.push(prompt);
            let output = llm.generate(his, stop.clone()).await;
            (output.text[0].content.clone(), output.info)
        });

        let res = futures::future::join_all(futs).await;
        println!("{:#?}", res);

        let futs_rerank = res.iter().map(|(answer, _)| async {
            let mut his = Vec::new();
            let prompt = self
                .prepare_prompt(&btreemap! {
                    "question".to_string() => input["question"].clone(),
                    "answer".to_string() => answer.clone(),
                })
                .unwrap();
            his.push(prompt);
            let output = llm.generate(his, stop.clone()).await;
            (output.text[0].content.clone(), output.info)
        });

        let res_rerank = futures::future::join_all(futs_rerank).await;
        println!("{:#?}", res_rerank);

        let jsons = res_rerank
            .iter()
            .map(|(answer, info)| {
                let json = serde_json::from_str::<serde_json::Value>(answer).unwrap();
                json
            })
            .collect_vec();

        let max_score = jsons
            .iter()
            .max_by(|a, b| {
                a["score"]
                    .as_f64()
                    .unwrap()
                    .partial_cmp(&b["score"].as_f64().unwrap())
                    .unwrap()
            })
            .unwrap();
        Some(Generation {
            text: vec![Message {
                role: "assistant".to_string(),
                content: serde_json::to_string(max_score).unwrap(),
            }],
            info: None,
        })
    }
}

#[tokio::test]
async fn test_map_reduce() {
    use crate::chain::llm_chain::LLMChain;
    dotenvy::dotenv().unwrap();

    let map_chain = LLMChain::new(Some(PromptTemplate::from("{question}".to_string())));

    let inputs = btreemap! {
        "question".to_string() => "what is a computer program".to_string(),
        "1".to_string() => r#"{"question": "What is human?"}"#.to_string(),
        "2".to_string() => r#"{"question": "What is iphone?"}"#.to_string(),
        "3".to_string() => r#"{"question": "What is programmer?"}"#.to_string(),
    };

    let executor = GLMClient::default();
    let chain = MapRerankChain {
        prompt_template: None,
        map_chain,
    };

    let res = chain.generate(None, &executor, &inputs, vec![]).await;
    println!("{:#?}", res);
}
