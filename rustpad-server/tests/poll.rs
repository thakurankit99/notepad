//! Tests for the HTTP long-polling fallback transport.
//!
//! These exercise the `connect` / `poll` / `send` endpoints that let clients
//! behind proxies blocking the WebSocket upgrade still collaborate, including
//! convergence with a regular WebSocket client on the same document.

use anyhow::Result;
use common::*;
use operational_transform::OperationSeq;
use rustpad_server::{server, ServerConfig};
use serde_json::{json, Value};

pub mod common;

/// POST `/api/connect/{id}`, returning the session token and initial messages.
async fn poll_connect(
    filter: &warp::filters::BoxedFilter<(impl warp::Reply + 'static,)>,
    id: &str,
) -> Result<(String, Value)> {
    let resp = warp::test::request()
        .method("POST")
        .path(&format!("/api/connect/{}", id))
        .reply(filter)
        .await;
    assert_eq!(resp.status(), 200);
    let body: Value = serde_json::from_slice(resp.body())?;
    let session = body["session"].as_str().unwrap().to_string();
    Ok((session, body["messages"].clone()))
}

/// GET `/api/poll/{id}?session=...`, returning the array of new messages.
async fn poll_recv(
    filter: &warp::filters::BoxedFilter<(impl warp::Reply + 'static,)>,
    id: &str,
    session: &str,
) -> Result<Value> {
    let resp = warp::test::request()
        .path(&format!("/api/poll/{}?session={}", id, session))
        .reply(filter)
        .await;
    assert_eq!(resp.status(), 200);
    Ok(serde_json::from_slice(resp.body())?)
}

/// POST `/api/send/{id}?session=...` with a client message, returning the status.
async fn poll_send(
    filter: &warp::filters::BoxedFilter<(impl warp::Reply + 'static,)>,
    id: &str,
    session: &str,
    msg: &Value,
) -> u16 {
    warp::test::request()
        .method("POST")
        .path(&format!("/api/send/{}?session={}", id, session))
        .json(msg)
        .reply(filter)
        .await
        .status()
        .as_u16()
}

#[tokio::test]
async fn test_poll_single_operation() -> Result<()> {
    pretty_env_logger::try_init().ok();
    let filter = server(ServerConfig::default());

    expect_text(&filter, "polldoc", "").await;

    let (session, messages) = poll_connect(&filter, "polldoc").await?;
    assert_eq!(messages, json!([{ "Identity": 0 }]));

    let mut operation = OperationSeq::default();
    operation.insert("hello");
    let edit = json!({ "Edit": { "revision": 0, "operation": operation } });
    assert_eq!(poll_send(&filter, "polldoc", &session, &edit).await, 200);

    let messages = poll_recv(&filter, "polldoc", &session).await?;
    assert_eq!(
        messages,
        json!([{
            "History": {
                "start": 0,
                "operations": [{ "id": 0, "operation": ["hello"] }]
            }
        }])
    );

    expect_text(&filter, "polldoc", "hello").await;
    Ok(())
}

#[tokio::test]
async fn test_poll_unknown_session() -> Result<()> {
    pretty_env_logger::try_init().ok();
    let filter = server(ServerConfig::default());

    let resp = warp::test::request()
        .path("/api/poll/polldoc?session=deadbeefdeadbeef")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 409);

    let edit = json!({ "Edit": { "revision": 0, "operation": [] } });
    assert_eq!(
        poll_send(&filter, "polldoc", "deadbeefdeadbeef", &edit).await,
        409
    );
    Ok(())
}

#[tokio::test]
async fn test_poll_websocket_interop() -> Result<()> {
    pretty_env_logger::try_init().ok();
    let filter = server(ServerConfig::default());

    // A regular WebSocket client connects first (user id 0).
    let mut ws = connect(&filter, "mixed").await?;
    assert_eq!(ws.recv().await?, json!({ "Identity": 0 }));

    // A long-polling client connects second (user id 1).
    let (session, messages) = poll_connect(&filter, "mixed").await?;
    assert_eq!(messages, json!([{ "Identity": 1 }]));

    // The WebSocket client inserts "A".
    let mut operation = OperationSeq::default();
    operation.insert("A");
    ws.send(&json!({ "Edit": { "revision": 0, "operation": operation } }))
        .await;
    assert_eq!(
        ws.recv().await?,
        json!({
            "History": { "start": 0, "operations": [{ "id": 0, "operation": ["A"] }] }
        })
    );

    // The polling client observes that edit.
    let messages = poll_recv(&filter, "mixed", &session).await?;
    assert_eq!(
        messages,
        json!([{
            "History": { "start": 0, "operations": [{ "id": 0, "operation": ["A"] }] }
        }])
    );

    // The polling client appends "B" via the send endpoint.
    let mut operation = OperationSeq::default();
    operation.retain(1);
    operation.insert("B");
    let edit = json!({ "Edit": { "revision": 1, "operation": operation } });
    assert_eq!(poll_send(&filter, "mixed", &session, &edit).await, 200);

    // The WebSocket client observes the polling client's edit.
    assert_eq!(
        ws.recv().await?,
        json!({
            "History": { "start": 1, "operations": [{ "id": 1, "operation": [1, "B"] }] }
        })
    );

    expect_text(&filter, "mixed", "AB").await;
    Ok(())
}
