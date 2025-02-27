use crate::persistence::Target;
use processmanager::{
    ProcessControlHandler, ProcessOperation, Runnable, RuntimeError, RuntimeGuard,
};
use std::time::Duration;

pub(super) struct PingMonitor {
    target: Target,
    runtime_guard: RuntimeGuard,
}

impl PingMonitor {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            runtime_guard: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl Runnable for PingMonitor {
    async fn process_start(&self) -> Result<(), RuntimeError> {
        let mut interval = tokio::time::interval(Duration::from_secs(3));

        let ticker = self.runtime_guard.runtime_ticker().await;
        loop {
            match ticker.tick(interval.tick()).await {
                ProcessOperation::Next(_) => {
                    println!("Ping: {}/{}", self.target.id, self.target.address);
                }
                ProcessOperation::Control(_) => break,
            }
        }

        Ok(())
    }

    fn process_handle(&self) -> Box<dyn ProcessControlHandler> {
        Box::new(self.runtime_guard.handle())
    }
}
