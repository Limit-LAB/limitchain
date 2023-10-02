use colored::Colorize;
use std::str::FromStr;

use limitchain::{
    btreemap,
    chain::{Chain, InMemMemory, Memory},
    schema::Message,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    // create memory
    let mut memory: Box<dyn Memory + Send + Sync> = Box::new(InMemMemory::from(vec![]));
    // print help message
    println!("{}", "Type `:help` to see the help message".green());
    loop {
        // read input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();

        match input.as_str() {
            ":h" | ":help" => {
                println!("{}", "Type `:q` or `:quit` to quit".green());
                println!("{}", "Type `:h` or `:help` to see the help message".green());
                println!("{}", "Type `:c` or `:clear` to clear the memory".green());
                println!("{}", "Type `:m` or `:memory` to see the memory".green());
                println!("{}", "Type `:u` or `:undo` to undo the memory".green());
            }
            ":q" | ":quit" => {
                break;
            }
            ":c" | ":clear" => {
                memory = Box::new(InMemMemory::from(vec![]));
            }
            ":m" | ":memory" => {
                println!("{:#?}", memory.get_history().await.unwrap());
            }
            ":u" | ":undo" => {
                memory.pop_back().await.unwrap();
                memory.pop_back().await.unwrap();
            }
            _ => {
                // prepare chain
                let chain = limitchain::chain::llm_chain::LLMChain::new(None);
                // prepare executor
                let executor = limitchain::client::glm::GLMClient::default();
                // prepare inputs
                let inputs = btreemap! {
                    "question".to_string() => input.trim().to_string()
                };
                // execute chain
                let res = chain
                    .generate(Some(&memory), &executor, &inputs, vec![])
                    .await;
                // print result
                println!("{}", res.clone().unwrap().text[0].content.blue().bold());
                // update memory
                memory
                    .push_back(Message::from_str(&format!("user: {}", input)).unwrap())
                    .await
                    .unwrap();
                memory
                    .push_back(res.unwrap().text[0].clone())
                    .await
                    .unwrap();
            }
        }
    }
}
