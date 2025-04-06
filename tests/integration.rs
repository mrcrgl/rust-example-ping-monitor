use processmanager::Runnable;
use serde::{Deserialize, Serialize};

#[tokio::test]
async fn test_rest() {
    let process = ping_monitor_rs::create_process().await;

    let handle = process.process_handle();

    tokio::spawn(async move {
        let _ = process.process_start().await;
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let probe_in = NewProbePayload {
        addr: "1.2.3.4".to_string(),
    };
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3000/targets")
        .json(&probe_in)
        .send()
        .await
        .unwrap();
    let res_body: ProbePayload = response.json().await.unwrap();

    assert_eq!(res_body.address, probe_in.addr);

    let response = client
        .get(format!("http://localhost:3000/targets/{}", res_body.id))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response = client
        .get(format!(
            "http://localhost:3000/targets/{}/results",
            res_body.id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response = client
        .delete(format!("http://localhost:3000/targets/{}", res_body.id))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);

    let response = client
        .get(format!("http://localhost:3000/targets/{}", res_body.id))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    handle.shutdown().await;
}

#[derive(Serialize)]
struct NewProbePayload {
    addr: String,
}

#[derive(Deserialize)]
struct ProbePayload {
    id: String,
    address: String,
}
