use crate::monitor::ping::PingMonitor;
use crate::persistence::{SharedStateTargetDatabase, Target, TargetId, Update};
use processmanager::{ProcessControlHandler, Runnable, RuntimeError, RuntimeGuard};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MonitorManager {
    database: Arc<SharedStateTargetDatabase>,
    runtime_guard: RuntimeGuard,
    monitors: RwLock<HashMap<TargetId, Box<dyn ProcessControlHandler>>>,
}

impl MonitorManager {
    pub fn new(database: Arc<SharedStateTargetDatabase>) -> Self {
        Self {
            database,
            runtime_guard: Default::default(),
            monitors: Default::default(),
        }
    }

    async fn start_monitor(&self, target: Target) {
        let id = target.id;
        let monitor = PingMonitor::new(target);

        let handle = monitor.process_handle();

        tokio::spawn(async move { monitor.process_start().await });

        let mut guard = self.monitors.write().await;
        guard.insert(id, handle);
    }
}

#[async_trait::async_trait]
impl Runnable for MonitorManager {
    async fn process_start(&self) -> Result<(), RuntimeError> {
        let mut updates = self.database.subscribe();

        while let Ok(update) = updates.recv().await {
            println!("update: {:?}", update);
            match update {
                Update::Deleted(_) => {
                    eprintln!("delete not implemented");
                }
                Update::Updated(target) => {
                    self.start_monitor(target).await;
                }
            }
        }

        Ok(())
    }

    fn process_handle(&self) -> Box<dyn ProcessControlHandler> {
        Box::new(self.runtime_guard.handle())
    }
}
