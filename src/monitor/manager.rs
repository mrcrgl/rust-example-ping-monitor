use crate::monitor::ping::PingMonitor;
use crate::persistence::{SharedStateTargetDatabase, Target, TargetId, TargetProbeResult, Update};
use processmanager::{
    ProcessControlHandler, ProcessOperation, Runnable, RuntimeError, RuntimeGuard,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::interval;

pub struct MonitorManager {
    database: Arc<SharedStateTargetDatabase>,
    runtime_guard: RuntimeGuard,
    monitors: RwLock<HashMap<TargetId, Box<dyn ProcessControlHandler>>>,
    result_ch_tx: mpsc::Sender<(TargetId, TargetProbeResult)>,
    result_ch_rx: Mutex<mpsc::Receiver<(TargetId, TargetProbeResult)>>,
}

impl MonitorManager {
    pub fn new(database: Arc<SharedStateTargetDatabase>) -> Self {
        let (result_ch_tx, result_ch_rx) = mpsc::channel(15);
        Self {
            database,
            runtime_guard: Default::default(),
            monitors: Default::default(),
            result_ch_rx: Mutex::new(result_ch_rx),
            result_ch_tx,
        }
    }

    async fn start_monitor(&self, target: Target) {
        let id = target.id;
        let monitor = PingMonitor::new(target, None, self.result_ch_tx.clone());

        let handle = monitor.process_handle();

        tokio::spawn(async move { monitor.process_start().await });

        let mut guard = self.monitors.write().await;
        guard.insert(id, handle);
    }

    /// Stop a certain process identified by TargetId.
    async fn stop_monitor(&self, target_id: TargetId) {
        let mut guard = self.monitors.write().await;

        if let Some(handle) = guard.get(&target_id) {
            handle.shutdown().await;

            guard.remove(&target_id);
        }
    }

    async fn handle_database_event(&self, event: Update) {
        match event {
            Update::Deleted(target) => {
                self.stop_monitor(target).await;
            }
            Update::Updated(target) => {
                self.start_monitor(target).await;
            }
        }
    }

    async fn handle_result_event(&self, target_id: TargetId, result: TargetProbeResult) {
        self.database.push_target_result(target_id, result).await;
    }
}

#[async_trait::async_trait]
impl Runnable for MonitorManager {
    async fn process_start(&self) -> Result<(), RuntimeError> {
        let mut updates = self.database.subscribe();
        let mut result_ch_rx = self.result_ch_rx.lock().await;

        let mut int = interval(tokio::time::Duration::from_millis(10));
        let ticker = self.runtime_guard.runtime_ticker().await;

        // ProcessOperation::Next indicates, that the process should continue to run.
        // When a graceful shutdown is initiated, the value changes as the loop.
        while let ProcessOperation::Next(_) = ticker.tick(int.tick()).await {
            tokio::select! {
                Ok(event) = updates.recv() => {
                    self.handle_database_event(event).await;
                }
                Some(event) = result_ch_rx.recv() => {
                    self.handle_result_event(event.0, event.1).await;
                }
                else => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn process_handle(&self) -> Box<dyn ProcessControlHandler> {
        Box::new(self.runtime_guard.handle())
    }
}
