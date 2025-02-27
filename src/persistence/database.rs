use crate::persistence::models::{Target, TargetId};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone)]
pub enum Update {
    Deleted(TargetId),
    Updated(Target),
}

pub struct SharedStateTargetDatabase {
    inner: RwLock<HashMap<TargetId, Target>>,
    ch_sender: broadcast::Sender<Update>,
}

impl SharedStateTargetDatabase {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(15);

        Self {
            inner: RwLock::new(HashMap::new()),
            ch_sender: sender,
        }
    }

    pub async fn insert(&self, target: Target) -> TargetId {
        let id = target.id;
        let mut guard = self.inner.write().await;
        guard.insert(id, target.clone());
        self.ch_sender
            .send(Update::Updated(target))
            .expect("send update");

        id
    }

    pub async fn list_keys(&self) -> Vec<TargetId> {
        self.inner.read().await.keys().cloned().collect()
    }

    pub async fn get(&self, key: &TargetId) -> Option<Target> {
        let handle = self.inner.read().await;
        handle.get(key).cloned()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Update> {
        self.ch_sender.subscribe()
    }
}
