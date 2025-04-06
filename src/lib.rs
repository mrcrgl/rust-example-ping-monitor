use crate::api::ApiServer;
use crate::monitor::MonitorManager;
use crate::persistence::SharedStateTargetDatabase;
use processmanager::receiver::SignalReceiver;
use processmanager::{ProcessManager, Runnable};
use std::sync::Arc;

mod api;
mod monitor;
pub(crate) mod persistence;

pub async fn create_process() -> impl Runnable {
    let database = Arc::new(SharedStateTargetDatabase::new());
    let monitor_manager = MonitorManager::new(Arc::clone(&database));
    let server = ApiServer::new(Arc::clone(&database));

    let mut process = ProcessManager::new();
    process.insert(SignalReceiver::default());
    process.insert(server);
    process.insert(monitor_manager);

    process
}
