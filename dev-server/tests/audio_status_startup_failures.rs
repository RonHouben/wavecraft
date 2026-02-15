//! Integration tests for startup-failure audio status broadcasting.
//!
//! These tests validate that forced startup-failure diagnostics are surfaced as
//! explicit `failed` runtime status and broadcast to connected clients via the
//! `audioStatusChanged` notification.

use futures_util::StreamExt;
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use wavecraft_bridge::{IpcHandler, ParameterHost};
use wavecraft_dev_server::{DevServerHost, WsServer, audio_status_with_diagnostic};
use wavecraft_protocol::{
    AudioDiagnosticCode, AudioRuntimePhase, AudioRuntimeStatus, IpcNotification,
    NOTIFICATION_AUDIO_STATUS_CHANGED,
};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("read local addr").port();
    drop(listener);
    port
}

async fn connect_client(
    port: u16,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://127.0.0.1:{port}");

    for _ in 0..20 {
        if let Ok((stream, _)) = connect_async(&url).await {
            return stream;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    panic!("failed to connect websocket client to {url}");
}

async fn assert_failed_status_broadcasted_to_connected_clients(code: AudioDiagnosticCode) {
    let host = Arc::new(DevServerHost::new(vec![]));
    let handler = Arc::new(IpcHandler::new(host.clone()));
    let port = free_port();
    let server = WsServer::new(port, handler);

    server.start().await.expect("start ws server");

    let mut client_a = connect_client(port).await;
    let mut client_b = connect_client(port).await;

    let status = audio_status_with_diagnostic(
        AudioRuntimePhase::Failed,
        code,
        format!("forced startup failure: {code:?}"),
        Some("test harness"),
        None,
        None,
    );

    host.set_audio_status(status.clone());
    server
        .handle()
        .broadcast_audio_status_changed(&status)
        .await
        .expect("broadcast audioStatusChanged");

    let msg_a = timeout(Duration::from_secs(2), client_a.next())
        .await
        .expect("client A timed out")
        .expect("client A stream closed")
        .expect("client A websocket error");

    let msg_b = timeout(Duration::from_secs(2), client_b.next())
        .await
        .expect("client B timed out")
        .expect("client B stream closed")
        .expect("client B websocket error");

    let text_a = match msg_a {
        Message::Text(text) => text,
        other => panic!("expected text message for client A, got: {other:?}"),
    };
    let text_b = match msg_b {
        Message::Text(text) => text,
        other => panic!("expected text message for client B, got: {other:?}"),
    };

    let notif_a: IpcNotification =
        serde_json::from_str(&text_a).expect("parse client A notification");
    let notif_b: IpcNotification =
        serde_json::from_str(&text_b).expect("parse client B notification");

    assert_eq!(notif_a.method, NOTIFICATION_AUDIO_STATUS_CHANGED);
    assert_eq!(notif_b.method, NOTIFICATION_AUDIO_STATUS_CHANGED);

    let status_a: AudioRuntimeStatus = serde_json::from_value(
        notif_a
            .params
            .expect("client A notification should contain params"),
    )
    .expect("deserialize audio runtime status for client A");
    let status_b: AudioRuntimeStatus = serde_json::from_value(
        notif_b
            .params
            .expect("client B notification should contain params"),
    )
    .expect("deserialize audio runtime status for client B");

    assert_eq!(status_a.phase, AudioRuntimePhase::Failed);
    assert_eq!(status_b.phase, AudioRuntimePhase::Failed);
    assert_eq!(
        status_a
            .diagnostic
            .as_ref()
            .expect("client A status should include diagnostic")
            .code,
        code
    );
    assert_eq!(
        status_b
            .diagnostic
            .as_ref()
            .expect("client B status should include diagnostic")
            .code,
        code
    );

    let host_status = host
        .get_audio_status()
        .expect("host should always return audio status");
    assert_eq!(host_status.phase, AudioRuntimePhase::Failed);
    assert_eq!(
        host_status
            .diagnostic
            .expect("host status should include diagnostic")
            .code,
        code
    );
}

#[tokio::test]
async fn audio_status_changed_broadcasts_failed_loader_unavailable() {
    assert_failed_status_broadcasted_to_connected_clients(AudioDiagnosticCode::LoaderUnavailable)
        .await;
}

#[tokio::test]
async fn audio_status_changed_broadcasts_failed_vtable_missing() {
    assert_failed_status_broadcasted_to_connected_clients(AudioDiagnosticCode::VtableMissing).await;
}

#[tokio::test]
async fn audio_status_changed_broadcasts_failed_processor_create_failure() {
    assert_failed_status_broadcasted_to_connected_clients(
        AudioDiagnosticCode::ProcessorCreateFailed,
    )
    .await;
}
