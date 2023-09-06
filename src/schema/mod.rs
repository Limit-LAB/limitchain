// pub mod text;
// pub mod chat_messages;
// pub mod examples;
// pub mod documents;

use std::{collections::BTreeMap, str::FromStr};

use serde::Serialize;

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

#[derive(Debug, Clone)]
pub struct Generation {
    pub text: Vec<Message>,
    pub info: Option<BTreeMap<String, String>>,
}
