use crate::persistence::{SharedStateTargetDatabase, Target, TargetId, TargetProbeResult};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Json};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;

pub(super) fn setup_router() -> axum::Router {
    axum::Router::new()
        .route("/targets", get(list_targets).post(add_target))
        .route("/targets/{:id}", get(get_target).delete(delete_target))
        .route("/targets/{:id}/results", get(get_target_results))
}

async fn list_targets(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
) -> Json<Vec<Target>> {
    let mut targets = vec![];

    for key in database.list_keys().await {
        if let Some(target) = database.get(&key).await {
            targets.push(target.target.clone());
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

async fn delete_target(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
    Path(id): Path<TargetId>,
) -> StatusCode {
    let _ = database.delete(id).await;

    StatusCode::ACCEPTED
}

async fn get_target(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
    Path(id): Path<TargetId>,
) -> axum::response::Result<Json<Target>> {
    let entry = database.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(entry.target))
}

async fn get_target_results(
    Extension(database): Extension<Arc<SharedStateTargetDatabase>>,
    Path(id): Path<TargetId>,
) -> axum::response::Result<Json<VecDeque<TargetProbeResult>>> {
    let entry = database.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(entry.probe_results))
}
