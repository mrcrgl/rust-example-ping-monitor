use crate::persistence::models::{Target, TargetId};
use crate::persistence::{TargetProbeResult, TargetTableEntry};
use std::collections::{HashMap, VecDeque};
use tokio::sync::{RwLock, broadcast};

static MAX_PROBES: usize = 300;

#[derive(Debug, Clone)]
pub enum Update {
    Deleted(TargetId),
    Updated(Target),
}

/// SharedStateTargetDatabase is an implementation of a simple key-value in-memory database.
///
/// # Features
///
/// - Its thread-safe
/// - Provides update channels for other parties to subscribe to updates
///
pub struct SharedStateTargetDatabase {
    inner: RwLock<HashMap<TargetId, TargetTableEntry>>,
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
        guard.insert(
            id,
            TargetTableEntry {
                target: target.clone(),
                probe_results: VecDeque::with_capacity(MAX_PROBES),
            },
        );
        let _ = self.ch_sender.send(Update::Updated(target));

        id
    }

    pub async fn delete(&self, target_id: TargetId) -> Option<Target> {
        let mut guard = self.inner.write().await;
        let entry = guard.remove(&target_id);
        let _ = self.ch_sender.send(Update::Deleted(target_id));

        entry.map(|target| target.target)
    }

    pub async fn push_target_result(&self, target_id: TargetId, result: TargetProbeResult) {
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get_mut(&target_id) {
            if entry.probe_results.len() == MAX_PROBES {
                entry.probe_results.pop_front();
            }
            entry.probe_results.push_back(result);
        }
    }

    pub async fn list_keys(&self) -> Vec<TargetId> {
        self.inner.read().await.keys().cloned().collect()
    }

    pub async fn get(&self, key: &TargetId) -> Option<TargetTableEntry> {
        let handle = self.inner.read().await;
        handle.get(key).cloned()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Update> {
        self.ch_sender.subscribe()
    }
}

impl Default for SharedStateTargetDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::ProbeStatus;
    use rand::Rng;
    use std::net::IpAddr;
    use std::str::FromStr;

    fn sample_target(id: &str) -> Target {
        Target {
            id: TargetId::parse_str(id).expect("valid target id"),
            address: IpAddr::from_str("8.8.8.8").unwrap(),
        }
    }

    fn sample_result() -> TargetProbeResult {
        let mut rng = rand::rng();
        TargetProbeResult {
            probe_time: chrono::Utc::now(),
            took: rng.random(),
            result: ProbeStatus::Ok { rtt: rng.random() },
        }
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let db = SharedStateTargetDatabase::new();
        let target = sample_target("d47c5e05-4785-46fe-b439-ba9ec0c320cb");

        let id = db.insert(target.clone()).await;
        let entry = db.get(&id).await;

        assert!(entry.is_some());
        assert_eq!(entry.unwrap().target.id, target.id);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = SharedStateTargetDatabase::new();
        let target = sample_target("d47c5e05-4785-46fe-b439-ba9ec0c320cb");

        let id = db.insert(target.clone()).await;
        let deleted = db.delete(id).await;

        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().id, target.id);

        let entry = db.get(&id).await;
        assert!(entry.is_none());
    }

    #[tokio::test]
    async fn test_push_probe_result_and_ring_buffer_behavior() {
        let db = SharedStateTargetDatabase::new();
        let target = sample_target("d47c5e05-4785-46fe-b439-ba9ec0c320cb");
        let id = db.insert(target).await;

        for _ in 0..(MAX_PROBES + 10) {
            db.push_target_result(id, sample_result()).await;
        }

        let entry = db.get(&id).await.unwrap();
        assert_eq!(entry.probe_results.len(), MAX_PROBES);
    }

    #[tokio::test]
    async fn test_list_keys() {
        let db = SharedStateTargetDatabase::new();

        let target1 = sample_target("d47c5e05-4785-46fe-b439-aaaaaaaaaaaa");
        let target2 = sample_target("d47c5e05-4785-46fe-b439-bbbbbbbbbbbb");

        db.insert(target1.clone()).await;
        db.insert(target2.clone()).await;

        let keys = db.list_keys().await;

        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&target1.id));
        assert!(keys.contains(&target2.id));
    }

    #[tokio::test]
    async fn test_subscription_broadcasts() {
        let db = SharedStateTargetDatabase::new();
        let mut sub = db.subscribe();

        let target = sample_target("d47c5e05-4785-46fe-b439-ba9ec0c320cb");
        let id = db.insert(target.clone()).await;

        match sub.recv().await {
            Ok(Update::Updated(t)) => assert_eq!(t.id, id),
            _ => panic!("Expected Updated event"),
        }

        db.delete(id).await;

        match sub.recv().await {
            Ok(Update::Deleted(did)) => assert_eq!(did, id),
            _ => panic!("Expected Deleted event"),
        }
    }
}
