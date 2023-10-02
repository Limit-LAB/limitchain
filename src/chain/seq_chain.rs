use itertools::Itertools;
use serde::Deserialize;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "chain_type")]
pub struct SeqChain<Chain1: Chain + Serialize, Chain2: Chain + Serialize> {
    prompt_template: Option<PromptTemplate>,
    chain1: Chain1,
    chain2: Chain2,
}

impl<Chain1: Chain, Chain2: Chain> SeqChain<Chain1, Chain2> {
    pub fn new(prompt_template: Option<PromptTemplate>, chain1: Chain1, chain2: Chain2) -> Self {
        Self {
            prompt_template,
            chain1,
            chain2,
        }
    }
}

#[async_trait::async_trait]
impl<Chain1: Chain + Serialize + Send + Sync, Chain2: Chain + Serialize + Send + Sync> Chain
    for SeqChain<Chain1, Chain2>
{
    fn get_input_keys(&self) -> Vec<String> {
        let mut chain2_input_keys = self.chain2.get_input_keys();
        chain2_input_keys.append(&mut self.chain1.get_input_keys());
        chain2_input_keys
    }

    fn get_output_keys(&self) -> Vec<String> {
        self.chain2.get_output_keys()
    }

    fn get_prompt_template(&self) -> PromptTemplate {
        const DEFAULT_JOIN_TEMPLATE: &str = r"
background:
{previous_output}

question:
{question}
";

        self.prompt_template
            .clone()
            .unwrap_or_else(|| PromptTemplate::from(DEFAULT_JOIN_TEMPLATE.to_string()))
    }

    async fn generate(
        &self,
        memory: Option<&Box<dyn Memory + Send + Sync>>,
        llm: &impl LLM,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<Generation> {
        let previous_output = self
            .chain1
            .generate(memory, llm, input, stop.clone())
            .await?;
        let question = self.chain2.prepare_prompt(input)?;
        let prompt = self
            .get_prompt_template()
            .format(&BTreeMap::from_iter(vec![
                (
                    "previous_output".to_string(),
                    previous_output.text[0].content.clone(),
                ),
                ("question".to_string(), question.content),
            ]))?;

        // debug
        println!("prompt: {}", prompt);

        let mut prompt = vec![
            // Message::from_str("system: using background to answer the following question").unwrap(),
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        match memory {
            Some(mem) => {
                let mut his = mem.get_history().await.unwrap();
                his.append(&mut prompt);
                Some(llm.generate(his, stop).await)
            }
            None => Some(llm.generate(prompt, stop).await),
        }
    }

    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>> {
        self.chain2.create_output(generation)
    }
}

#[tokio::test]
async fn test_bind_chain() {
    use super::llm_chain::*;
    use crate::btreemap;
    use crate::client::openai::*;
    dotenvy::dotenv().unwrap();

    let question1: &str =
        r"what does LGTM means in git issues? and where it comes from? who is the author?";
    let question2: &str = r"write a brief summary of that in chinese";
    let executor = OpenAIClient::default();
    let chain1 = LLMChain::new(Some(PromptTemplate::from("{question1}".to_string())));
    let chain2 = LLMChain::new(Some(PromptTemplate::from("{question2}".to_string())));
    let chain = SeqChain::new(None, chain1, chain2);
    println!("{:#?}", serde_json::to_string(&chain).unwrap());

    let res = chain
        .apply(
            None,
            &executor,
            &btreemap! {
                "question1".to_string() => question1.to_string(),
                "question2".to_string() => question2.to_string(),
            },
            vec!["stop".to_string()],
        )
        .await;

    println!("{:?}", res);
    chain
        .serialize(&mut serde_json::Serializer::new(&mut std::io::stdout()))
        .unwrap();
}

#[test]
fn serde_deserde() {
    use super::llm_chain::*;
    use crate::btreemap;
    use crate::client::openai::*;
    dotenvy::dotenv().unwrap();

    let question1: &str =
        r"what does LGTM means in git issues? and where it comes from? who is the author?";
    let question2: &str = r"write a brief summary of that in chinese";
    let executor = OpenAIClient::default();
    let chain1 = LLMChain::new(Some(PromptTemplate::from("{question1}".to_string())));
    let chain2 = LLMChain::new(Some(PromptTemplate::from("{question2}".to_string())));
    let chain = SeqChain::new(None, chain1, chain2);

    let str = serde_json::to_string(&chain).unwrap();
    println!("{:#?}", str);

    let chain: SeqChain<LLMChain, LLMChain> = serde_json::from_str(&str).unwrap();
    println!("{:#?}", chain);
}
