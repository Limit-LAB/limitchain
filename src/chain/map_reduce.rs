use std::collections::BTreeMap;

use crate::{
    chain::Chain,
    prompt_template::PromptTemplate,
    schema::{Generation, Message},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{Memory, LLM};

/// usage: {1: {q1}, 2: {q2}} -> {output}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "chain_type")]
pub struct MapReduceChain<
    MapChain: Chain + Serialize + Send + Sync,
    ReduceChain: Chain + Serialize + Send + Sync,
> {
    map_chain: MapChain,
    reduce_chain: ReduceChain,
}

#[async_trait::async_trait]
impl<MapChain: Chain + Serialize + Send + Sync, ReduceChain: Chain + Serialize + Send + Sync> Chain
    for MapReduceChain<MapChain, ReduceChain>
{
    fn get_input_keys(&self) -> Vec<String> {
        self.map_chain.get_input_keys()
    }

    fn get_output_keys(&self) -> Vec<String> {
        self.reduce_chain.get_output_keys()
    }

    fn get_prompt_template(&self) -> PromptTemplate {
        self.map_chain.get_prompt_template()
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
        let mut prompt = self.reduce_chain.prepare_prompt(&input).unwrap();
        prompt.content = format!(
            "{}\n{}",
            prompt.content,
            res.iter().map(|i| &i.0).join("\n")
        );

        let mut his = Vec::new();
        if let Some(mem) = memory {
            his = mem.get_history().await.unwrap();
        }
        his.push(prompt);
        Some(llm.generate(his, stop).await)
    }
}

#[tokio::test]
async fn test_map_reduce() {
    use crate::chain::llm_chain::LLMChain;
    use crate::btreemap;
    use crate::llm::client::glm::GLMClient;
    dotenvy::dotenv().unwrap();

    let map_chain = LLMChain::new(Some(PromptTemplate::from("{question}".to_string())));

    let inputs = btreemap! {
        "question".to_string() => "你能把下面的信息联系起来写一篇文章吗:".to_string(),
        "1".to_string() => r#"{"question": "What is human?"}"#.to_string(),
        "2".to_string() => r#"{"question": "What is computer?"}"#.to_string(),
        "3".to_string() => r#"{"question": "What is programmer?"}"#.to_string(),
    };

    let reduce_chain = LLMChain::new(Some(PromptTemplate::from("{question}".to_string())));

    let executor = GLMClient::default();
    let chain = MapReduceChain {
        map_chain,
        reduce_chain,
    };

    let res = chain.generate(None, &executor, &inputs, vec![]).await;
    println!("{:#?}", res);
}
