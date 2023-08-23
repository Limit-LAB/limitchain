// pub mod text;
// pub mod chat_messages;
// pub mod examples;
// pub mod documents;

use std::collections::BTreeMap;

pub struct Generation {
    pub text: Vec<String>,
    pub info: Option<BTreeMap<String, String>>,
}
