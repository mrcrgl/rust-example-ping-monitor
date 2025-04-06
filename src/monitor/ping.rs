use crate::persistence::{ProbeStatus, Target, TargetId, TargetProbeResult};
use chrono::Duration;
use ping_rs::{PingError, PingOptions, send_ping_async};
use processmanager::{
    ProcessControlHandler, ProcessOperation, Runnable, RuntimeError, RuntimeGuard,
};
use std::sync::Arc;
use tokio::sync::mpsc;

pub(super) struct PingMonitor {
    target: Target,
    timeout: Option<Duration>,
    runtime_guard: RuntimeGuard,
    result_sink: mpsc::Sender<(TargetId, TargetProbeResult)>,
}

impl PingMonitor {
    pub fn new(
        target: Target,
        timeout: Option<Duration>,
        result_sink: mpsc::Sender<(TargetId, TargetProbeResult)>,
    ) -> Self {
        Self {
            target,
            runtime_guard: Default::default(),
            timeout,
            result_sink,
        }
    }
}

#[async_trait::async_trait]
impl Runnable for PingMonitor {
    async fn process_start(&self) -> Result<(), RuntimeError> {
        let mut interval =
            tokio::time::interval(Duration::seconds(1).to_std().expect("invalid duration"));

        let ticker = self.runtime_guard.runtime_ticker().await;
        let options = PingOptions {
            ttl: 128,
            dont_fragment: true,
        };
        let timeout = self
            .timeout
            .unwrap_or(Duration::seconds(1))
            .to_std()
            .expect("invalid duration");
        let data: Vec<u8> = (0..16).collect();
        let data = Arc::new(&data[..]);

        while let ProcessOperation::Next(_) = ticker.tick(interval.tick()).await {
            let start = chrono::Utc::now();
            let status =
                match send_ping_async(&self.target.address, timeout, data.clone(), Some(&options))
                    .await
                {
                    Ok(value) => ProbeStatus::Ok {
                        rtt: value.rtt as i64,
                    },
                    Err(PingError::TimedOut) => ProbeStatus::Timeout,
                    Err(err) => ProbeStatus::Failure {
                        reason: format!("{err:?}"),
                    },
                };

            let result = TargetProbeResult {
                probe_time: start,
                took: (chrono::Utc::now() - start).num_milliseconds(),
                result: status,
            };

            tracing::debug!(
                "ping id={} target={} result={result:?}",
                self.target.id,
                self.target.address
            );

            self.result_sink
                .send((self.target.id, result))
                .await
                .expect("failed to send result to sink");
        }

        Ok(())
    }

    fn process_handle(&self) -> Box<dyn ProcessControlHandler> {
        Box::new(self.runtime_guard.handle())
    }
}
