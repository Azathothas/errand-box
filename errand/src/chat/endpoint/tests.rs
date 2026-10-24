//! The stand-in's guard, and serenity's handshake against the stand-in.
//!
//! The guard's tests each break one condition and name it; a correct pair is
//! taken. The handshake test builds the client through [`Endpoint`], as
//! `serve.rs` does, and reads what serenity sent from the stand-in's journal.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use discord_mock::{Config, Entry, Mock};
use serenity::client::EventHandler;
use serenity::model::id::ApplicationId;

use super::{Endpoint, MOCK_VARIABLE, Refused, check_gateway};
use crate::chat::gateway::Gateway;
use crate::config::load::Environment;

const TOKEN: &str = "mock.bot-token";

fn naming(url: &str) -> Environment {
    BTreeMap::from([(MOCK_VARIABLE.to_owned(), url.to_owned())])
}

#[test]
fn without_the_variable_the_client_is_discords() {
    assert_eq!(
        Endpoint::from_environment(&BTreeMap::new(), "MTIz.a-real-shape.token"),
        Ok(Endpoint::Discord)
    );
    assert_eq!(Endpoint::Discord.warning(), None);
}

#[test]
fn a_loopback_address_and_a_mock_token_are_taken() {
    for (url, base) in [
        ("http://127.0.0.1:4100", "http://127.0.0.1:4100"),
        ("http://127.0.0.1:4100/", "http://127.0.0.1:4100"),
        ("http://127.20.30.40:9", "http://127.20.30.40:9"),
        ("http://[::1]:4100", "http://[::1]:4100"),
    ] {
        assert_eq!(
            Endpoint::from_environment(&naming(url), TOKEN),
            Ok(Endpoint::Mock {
                base: base.to_owned()
            }),
            "{url}"
        );
    }
    let warning = Endpoint::Mock {
        base: "http://127.0.0.1:4100".into(),
    }
    .warning()
    .expect("a warning while the stand-in is in force");
    assert!(warning.starts_with("WARNING: "));
    assert!(warning.contains(MOCK_VARIABLE));
}

#[test]
fn an_address_off_loopback_is_refused() {
    for url in ["http://10.0.0.1:4100", "http://192.0.2.1", "http://[::2]:1"] {
        assert!(
            matches!(
                Endpoint::from_environment(&naming(url), TOKEN),
                Err(Refused::NotLoopback(_))
            ),
            "{url}"
        );
    }
}

#[test]
fn a_host_name_is_refused_even_one_that_means_loopback() {
    for url in [
        "http://localhost:4100",
        "http://discord.com",
        "http://127.0.0.1.example:1",
    ] {
        assert!(
            matches!(
                Endpoint::from_environment(&naming(url), TOKEN),
                Err(Refused::NotLoopback(_))
            ),
            "{url}"
        );
    }
}

#[test]
fn https_and_other_schemes_are_refused() {
    for url in [
        "https://127.0.0.1:4100",
        "ws://127.0.0.1:4100",
        "ftp://127.0.0.1",
    ] {
        assert!(
            matches!(
                Endpoint::from_environment(&naming(url), TOKEN),
                Err(Refused::NotHttp(_))
            ),
            "{url}"
        );
    }
    assert!(matches!(
        Endpoint::from_environment(&naming("not a url"), TOKEN),
        Err(Refused::NotAUrl(_))
    ));
}

#[test]
fn a_user_a_path_or_a_query_is_refused() {
    for url in [
        "http://someone@127.0.0.1:4100",
        "http://127.0.0.1:4100/api",
        "http://127.0.0.1:4100/?x=1",
    ] {
        assert_eq!(
            Endpoint::from_environment(&naming(url), TOKEN),
            Err(Refused::NotBare),
            "{url}"
        );
    }
}

#[test]
fn a_token_without_the_mock_prefix_is_refused() {
    for token in ["MTIz.a-real-shape.token", "Bot mock.bot-token", "", "mock"] {
        assert_eq!(
            Endpoint::from_environment(&naming("http://127.0.0.1:4100"), token),
            Err(Refused::NotAMockToken),
            "{token:?}"
        );
    }
}

#[test]
fn a_gateway_off_loopback_is_refused() {
    for url in [
        "wss://gateway.discord.gg",
        "ws://gateway.discord.gg",
        "ws://localhost:4101",
        "ws://10.0.0.1:4101",
        "wss://127.0.0.1:4101",
        "ws://someone@127.0.0.1:4101",
    ] {
        assert!(check_gateway(url).is_err(), "{url}");
    }
    assert_eq!(check_gateway("ws://127.0.0.1:4101"), Ok(()));
    assert_eq!(check_gateway("ws://[::1]:4101"), Ok(()));
}

struct Quiet;

impl EventHandler for Quiet {}

/// serenity falls back to Discord's own gateway when `GET /gateway` fails.
/// With nothing listening, the lookup fails, and the guard must refuse the
/// fallback before a socket opens.
#[tokio::test]
async fn a_failed_gateway_lookup_is_refused_not_sent_to_discord() {
    // A port that was just free: bound, read, and released.
    let free = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = free.local_addr().expect("its address").port();
    drop(free);
    let endpoint = Endpoint::from_environment(&naming(&format!("http://127.0.0.1:{port}")), TOKEN)
        .expect("the guard takes a loopback stand-in");
    let client = endpoint
        .builder(TOKEN, Gateway::intents())
        .expect("a builder")
        .event_handler(Quiet)
        .await
        .expect("serenity builds a client even when the lookup fails");
    assert_eq!(
        endpoint.confirm(&client).await,
        Err(Refused::GatewayNotLocal("wss://gateway.discord.gg".into()))
    );
}

/// What a journal entry received, as JSON, for an IDENTIFY.
fn identify(entries: &[Entry]) -> Option<serde_json::Value> {
    entries.iter().find_map(|entry| match entry {
        Entry::Received { op: 2, payload } => Some(payload["d"].clone()),
        _ => None,
    })
}

#[tokio::test]
async fn the_handshake_is_serenitys_through_the_endpoint() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the stand-in starts");
    let token = mock.config().token.clone();
    let endpoint = Endpoint::from_environment(&naming(&mock.base_url), &token)
        .expect("the guard takes the stand-in");
    let mut client = endpoint
        .builder(&token, Gateway::intents())
        .expect("a builder")
        .event_handler(Quiet)
        .await
        .expect("a client");
    endpoint
        .confirm(&client)
        .await
        .expect("the stand-in's gateway is on loopback");
    let http = Arc::clone(&client.http);
    let running = tokio::spawn(async move {
        let _ = client.start().await;
    });

    let journal = mock
        .wait_for(Duration::from_secs(20), |entries| {
            entries
                .iter()
                .any(|entry| entry.sent_event() == Some("READY"))
                .then(|| entries.to_vec())
        })
        .await
        .expect("serenity identifies and the stand-in answers READY");

    // One REST call before the socket: the gateway lookup. Not
    // `/gateway/bot`, which only autosharding asks for.
    let connect = journal
        .iter()
        .position(|entry| matches!(entry, Entry::Connect { .. }))
        .expect("a socket");
    let before: Vec<&Entry> = journal[..connect]
        .iter()
        .filter(|entry| matches!(entry, Entry::Rest { .. }))
        .collect();
    assert_eq!(before.len(), 1, "{before:?}");
    assert!(before[0].is_call("GET", "/api/v10/gateway"), "{before:?}");
    assert_eq!(
        journal[connect],
        Entry::Connect {
            query: "v=10".into()
        }
    );

    let identify = identify(&journal).expect("an IDENTIFY");
    assert_eq!(identify["intents"], 33281);
    assert_eq!(identify["compress"], true);
    assert_eq!(identify["token"], format!("Bot {token}"));

    let ready = journal
        .iter()
        .find_map(|entry| match entry {
            Entry::Sent { payload, .. } if payload["t"] == "READY" => Some(payload.clone()),
            _ => None,
        })
        .expect("READY");
    assert_eq!(ready["s"], 1);

    // serenity learns the application id from READY.
    let want = ApplicationId::new(mock.config().application_id);
    let mut learned = None;
    for _ in 0..200 {
        learned = http.application_id();
        if learned.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(learned, Some(want));
    running.abort();
}

/// An application without the content intent's flag is closed with 4014,
/// and errand reads that as an answer that retrying cannot change, which is
/// what makes it exit "refused".
#[tokio::test]
async fn a_disallowed_content_intent_ends_the_client_with_a_permanent_error() {
    let mock = Mock::start(Config {
        message_content_intent: false,
        ..Config::default()
    })
    .await
    .expect("the stand-in starts");
    let token = mock.config().token.clone();
    let endpoint = Endpoint::from_environment(&naming(&mock.base_url), &token)
        .expect("the guard takes the stand-in");
    let mut client = endpoint
        .builder(&token, Gateway::intents())
        .expect("a builder")
        .event_handler(Quiet)
        .await
        .expect("a client");
    let ended = tokio::time::timeout(Duration::from_secs(20), client.start())
        .await
        .expect("serenity stops");
    let error = ended.expect_err("the gateway refuses the intents");
    assert!(Gateway::login_is_permanent(&error), "{error:?}");
    assert!(mock.journal().contains(&Entry::Closed {
        code: Some(4014),
        reason: "Disallowed intent(s).".into(),
    }));
}
