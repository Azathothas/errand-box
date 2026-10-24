//! Step 6 of the plan: the gateway's faults, against serenity's own client.
//!
//! Each test reads what serenity did from the journal: a heartbeat, a
//! RESUME, a second IDENTIFY. The pacing serenity keeps is its own: it waits
//! 5 seconds between IDENTIFYs (serenity 0.12.5
//! `src/gateway/bridge/shard_queuer.rs:35`, `:134-144`).

mod common;

use std::time::{Duration, Instant};

use serenity::Client;
use serenity::client::{ClientBuilder, EventHandler};
use serenity::model::gateway::GatewayIntents;
use tokio::task::JoinHandle;

use common::{ERRAND_INTENTS, http};
use discord_mock::{Config, Entry, Fault, Mock, MockHandle};

struct Quiet;

impl EventHandler for Quiet {}

/// serenity's client on the mock, started; returns once READY was sent.
async fn connected(mock: &MockHandle) -> JoinHandle<()> {
    let intents = GatewayIntents::from_bits_truncate(ERRAND_INTENTS);
    let mut client: Client = ClientBuilder::new_with_http(http(mock), intents)
        .event_handler(Quiet)
        .await
        .expect("a client");
    let running = tokio::spawn(async move {
        let _ = client.start().await;
    });
    mock.wait_for(Duration::from_secs(20), |entries| {
        entries
            .iter()
            .any(|entry| entry.sent_event() == Some("READY"))
            .then_some(())
    })
    .await
    .expect("READY");
    running
}

fn received(entries: &[Entry], op: u64) -> usize {
    entries
        .iter()
        .filter(|entry| matches!(entry, Entry::Received { op: got, .. } if *got == op))
        .count()
}

/// serenity #3575 reports that 0.12.5 fails on op 1 with `"s": null`. It
/// answers one here, so it parses.
#[tokio::test]
async fn a_heartbeat_request_with_a_null_sequence_is_answered() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let running = connected(&mock).await;
    let before = received(&mock.journal(), 1);
    mock.fault(Fault::RequestHeartbeat);
    mock.wait_for(Duration::from_secs(5), |entries| {
        (received(entries, 1) > before).then_some(())
    })
    .await
    .expect("a heartbeat answers op 1");
    running.abort();
}

#[tokio::test]
async fn a_reconnect_request_is_resumed_on_a_new_socket() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let running = connected(&mock).await;
    mock.fault(Fault::Reconnect);
    mock.wait_for(Duration::from_secs(10), |entries| {
        entries
            .iter()
            .any(|entry| entry.sent_event() == Some("RESUMED"))
            .then_some(())
    })
    .await
    .expect("RESUME, then RESUMED");
    let journal = mock.journal();
    assert_eq!(
        received(&journal, 2),
        1,
        "one IDENTIFY: the session was resumed"
    );
    assert_eq!(received(&journal, 6), 1);
    running.abort();
}

#[tokio::test]
async fn an_invalid_session_is_identified_again_no_sooner_than_five_seconds() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let running = connected(&mock).await;
    let ready_at = Instant::now();
    mock.fault(Fault::InvalidSession { resumable: false });
    mock.wait_for(Duration::from_secs(20), |entries| {
        (received(entries, 2) == 2).then_some(())
    })
    .await
    .expect("a second IDENTIFY");
    let waited = ready_at.elapsed();
    // Measured from READY, which follows the first start by a few
    // milliseconds, so the bound is a little under serenity's 5 seconds.
    assert!(waited >= Duration::from_millis(4500), "{waited:?}");
    assert_eq!(received(&mock.journal(), 6), 0, "not a RESUME");
    running.abort();
}

/// A heartbeat that is not acknowledged makes serenity start the shard
/// again, with a new IDENTIFY rather than a RESUME (serenity 0.12.5
/// `src/gateway/bridge/shard_runner.rs:112-117`).
#[tokio::test]
async fn a_withheld_ack_starts_a_new_session() {
    let mock = Mock::start(Config {
        heartbeat_interval_ms: 500,
        ..Config::default()
    })
    .await
    .expect("the mock starts");
    let running = connected(&mock).await;
    mock.fault(Fault::WithholdAcks(true));
    mock.wait_for(Duration::from_secs(20), |entries| {
        (received(entries, 2) == 2).then_some(())
    })
    .await
    .expect("a second IDENTIFY");
    assert_eq!(received(&mock.journal(), 6), 0, "not a RESUME");
    running.abort();
}

#[tokio::test]
async fn a_closed_socket_with_4004_ends_serenity_with_an_authentication_error() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let intents = GatewayIntents::from_bits_truncate(ERRAND_INTENTS);
    let mut client = ClientBuilder::new_with_http(http(&mock), intents)
        .event_handler(Quiet)
        .await
        .expect("a client");
    let running = tokio::spawn(async move { client.start().await });
    mock.wait_for(Duration::from_secs(20), |entries| {
        entries
            .iter()
            .any(|entry| entry.sent_event() == Some("READY"))
            .then_some(())
    })
    .await
    .expect("READY");
    mock.fault(Fault::Close(4004));
    let ended = tokio::time::timeout(Duration::from_secs(10), running)
        .await
        .expect("serenity stops")
        .expect("the task ends");
    assert!(
        matches!(
            ended,
            Err(serenity::Error::Gateway(
                serenity::gateway::GatewayError::InvalidAuthentication
            ))
        ),
        "{ended:?}"
    );
}
