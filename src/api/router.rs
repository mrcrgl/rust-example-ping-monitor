use crate::persistence::{SharedStateTargetDatabase, Target, TargetId};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Extension, Json};
use std::net::IpAddr;
use std::sync::Arc;

pub(super) fn setup_router() -> axum::Router {
    let router = axum::Router::new()
        .route("/targets", get(get_targets))
        .route("/targets", post(add_target));

    router
}

async fn get_targets(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
) -> Json<Vec<Target>> {
    let mut targets = vec![];

    for key in database.list_keys().await {
        if let Some(target) = database.get(&key).await {
            targets.push(target.clone());
        }
    }

    Json(targets)
}

#[derive(serde::Deserialize)]
struct AddTargetPayload {
    addr: String,
}

async fn add_target(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
    Json(payload): Json<AddTargetPayload>,
) -> Result<Json<Target>, StatusCode> {
    let addr: IpAddr = payload.addr.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let id = TargetId::new_v4();
    let target = Target { id, address: addr };

    database.insert(target.clone()).await;

    Ok(Json(target))
}
