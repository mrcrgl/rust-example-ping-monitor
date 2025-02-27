use crate::api::router::setup_router;
use crate::persistence::SharedStateTargetDatabase;
use async_trait::async_trait;
use axum::Extension;
use processmanager::{ProcessControlHandler, Runnable, RuntimeError};
use std::sync::Arc;

pub struct ApiServer {
    target_database: Arc<SharedStateTargetDatabase>,
    runtime_guard: processmanager::RuntimeGuard,
}

impl ApiServer {
    pub fn new(target_database: Arc<SharedStateTargetDatabase>) -> Self {
        Self {
            target_database,
            runtime_guard: Default::default(),
        }
    }
}

#[async_trait]
impl Runnable for ApiServer {
    async fn process_start(&self) -> Result<(), RuntimeError> {
        let target_database = Arc::clone(&self.target_database);

        let app = setup_router().layer(Extension(target_database));

        let addr = "0.0.0.0:3000";

        tracing::info!("Listening on {}", addr);

        let listener =
            tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|err| RuntimeError::Internal {
                    message: format!("Failed to open tcp channel: {err:?}"),
                })?;

        axum::serve(listener, app)
            .await
            .map_err(|err| RuntimeError::Internal {
                message: format!("Failed to start server: {err:?}"),
            })?;

        Ok(())
    }

    fn process_handle(&self) -> Box<dyn ProcessControlHandler> {
        Box::new(self.runtime_guard.handle())
    }
}
