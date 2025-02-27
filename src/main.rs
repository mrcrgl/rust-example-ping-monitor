use crate::api::ApiServer;
use crate::monitor::MonitorManager;
use crate::persistence::SharedStateTargetDatabase;
use processmanager::receiver::SignalReceiver;
use processmanager::{ProcessManager, Runnable};
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

mod api;
mod monitor;
pub mod persistence;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let database = Arc::new(SharedStateTargetDatabase::new());

    let monitor_manager = MonitorManager::new(Arc::clone(&database));
    let server = ApiServer::new(Arc::clone(&database));

    let mut process = ProcessManager::new();
    process.insert(SignalReceiver::default());
    process.insert(server);
    process.insert(monitor_manager);

    if let Err(err) = process.process_start().await {
        tracing::error!("Process failed: {:?}", err);
    }
}
