use std::collections::VecDeque;

use tokio::sync::Mutex;

use super::Message;


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
