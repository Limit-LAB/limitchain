// pub mod text;
// pub mod chat_messages;
// pub mod examples;
// pub mod documents;

use std::collections::BTreeMap;

use crate::chain::Message;

#[derive(Debug, Clone)]
pub struct Generation {
    pub text: Vec<Message>,
    pub info: Option<BTreeMap<String, String>>,
}
