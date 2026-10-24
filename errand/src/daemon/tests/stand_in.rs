//! The chat path against the Discord stand-in: a person's message reaches
//! the daemon through serenity and the gateway, and the session's thread is
//! made by the real `ChatThreadFactory`, as `serve.rs` wires them. Only the
//! sandbox is a fake.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use discord_mock::{Config as MockConfig, Entry, Mock, MockHandle};
use serde_json::{Value, json};
use serenity::model::id::ChannelId;
use tokio::sync::watch;

use super::{FakeSandbox, config_with, silent};
use crate::chat::commands::{TranslatedCommand, register_commands};
use crate::chat::endpoint::{Endpoint, MOCK_VARIABLE};
use crate::chat::gateway::{Gateway, GatewayHandlers, OnCommand, OnMessage};
use crate::chat::inbound::{InboundDecision, RawMessage};
use crate::chat::threads::ChatThreadFactory;
use crate::config::live::LiveConfig;
use crate::daemon::{Daemon, DaemonOptions, SlashCommand};
use crate::serve::{FactoryAdapter, GatewayHandler};
use crate::session::manager::{SandboxPool, ThreadFactory};
use crate::session::session::IncomingMessage;

/// A daemon serving the stand-in's channel, connected as `serve.rs`
/// connects it, with the stand-in's first person allowed.
struct Served {
    daemon: Arc<Daemon>,
    http: Arc<serenity::http::Http>,
    running: tokio::task::JoinHandle<()>,
    _root: tempfile::TempDir,
}

/// Where the gateway's handlers find the daemon, which exists only after
/// the connection is up, as in `serve.rs`.
type Holder = Arc<tokio::sync::Mutex<Option<Arc<Daemon>>>>;

/// A person's message goes to the daemon.
fn on_message(holder: &Holder) -> OnMessage {
    let holder = Arc::clone(holder);
    Arc::new(move |raw: RawMessage, decision: InboundDecision| {
        let holder = Arc::clone(&holder);
        tokio::spawn(async move {
            if let Some(daemon) = holder.lock().await.as_ref() {
                daemon.handle(raw, decision).await;
            }
        });
    })
}

/// As `serve.rs` answers a slash command: the daemon runs it, and the
/// acknowledgement edits the deferred answer in.
fn on_command(holder: &Holder) -> OnCommand {
    let holder = Arc::clone(holder);
    Arc::new(
        move |command: TranslatedCommand, ack: Arc<dyn Fn(&str) + Send + Sync>| {
            let holder = Arc::clone(&holder);
            tokio::spawn(async move {
                let daemon = holder.lock().await.clone();
                let answer = match daemon {
                    Some(daemon) => {
                        daemon
                            .run_command(&SlashCommand {
                                thread_id: command.thread_id.clone(),
                                user_id: command.user_id.clone(),
                                user_name: command.user_name.clone(),
                                content: command.content.clone(),
                            })
                            .await
                    }
                    None => "the daemon is not ready".to_owned(),
                };
                ack(&answer);
            });
        },
    )
}

async fn serve_the_stand_in(mock: &MockHandle) -> Served {
    let root = tempfile::tempdir().expect("a temp directory");
    let person = mock.config().people[0].id.to_string();
    let config = config_with(&json!({
        "chat": {
            "token": mock.config().token,
            "channelId": mock.config().channel_id.to_string(),
            "allowedUserIds": [person],
        },
        "projectRoot": root.path().join("projects").display().to_string(),
        "stateDir": root.path().join("state").display().to_string(),
    }));
    let holder: Holder = Arc::new(tokio::sync::Mutex::new(None));
    let gateway = Gateway::new(
        config.chat.clone(),
        GatewayHandlers {
            on_message: on_message(&holder),
            on_command: on_command(&holder),
            on_thread_closed: Arc::new(|_| {}),
            on_withdrawn: Arc::new(|_, _| {}),
            on_connected: Arc::new(|| {}),
            on_disconnected: Arc::new(|| {}),
            on_gave_up: Arc::new(|_| {}),
        },
        silent(),
    );
    let env = BTreeMap::from([(MOCK_VARIABLE.to_owned(), mock.base_url.clone())]);
    let endpoint =
        Endpoint::from_environment(&env, &config.chat.token).expect("the guard takes it");
    let mut client = endpoint
        .builder(&config.chat.token, Gateway::intents())
        .expect("a builder")
        .event_handler(GatewayHandler(Arc::clone(&gateway)))
        .await
        .expect("a client");
    endpoint.confirm(&client).await.expect("a loopback gateway");
    let http = Arc::clone(&client.http);
    let running = tokio::spawn(async move {
        let _ = client.start().await;
    });
    gateway.wait_ready(20_000).await.expect("READY");

    let (_connection, watching) = watch::channel(true);
    let factory = Arc::new(ChatThreadFactory::new(
        ChannelId::new(mock.config().channel_id),
        Arc::clone(&http),
        silent(),
        false,
        watching,
    ));
    let daemon = Arc::new(Daemon::new(DaemonOptions {
        config: LiveConfig::new(config),
        sandbox: FakeSandbox::new(None) as Arc<dyn SandboxPool>,
        threads: Arc::new(FactoryAdapter(factory)) as Arc<dyn ThreadFactory>,
        describe_images: None,
        log: silent(),
        reply_in_channel: Arc::new(|_: IncomingMessage, _: String| {
            Box::pin(async {}) as Pin<Box<dyn Future<Output = ()> + Send>>
        }),
        reply_with_file: Arc::new(|_: IncomingMessage, _: String, _: Vec<u8>, _: String| {
            Box::pin(async {}) as Pin<Box<dyn Future<Output = ()> + Send>>
        }),
        draw_image: None,
        memory: None,
        power_off: None,
        describe_usage: None,
        refresh_models: None,
        catalog: crate::provider::discover::Catalog::default(),
        delegate_base_url: None,
        operator_ids: None,
        unavailable: None,
    }));
    daemon.start(None).await.expect("the daemon starts");
    *holder.lock().await = Some(Arc::clone(&daemon));
    Served {
        daemon,
        http,
        running,
        _root: root,
    }
}

/// The index of the first entry that `matches`.
fn first(entries: &[Entry], matches: impl Fn(&Entry) -> bool) -> Option<usize> {
    entries.iter().position(matches)
}

/// The request body of a REST entry.
fn request_of(entry: &Entry) -> Value {
    match entry {
        Entry::Rest { request, .. } => request.clone(),
        _ => Value::Null,
    }
}

#[tokio::test]
async fn a_persons_message_opens_a_thread_on_the_stand_in() {
    let mock = Mock::start(MockConfig::default())
        .await
        .expect("the stand-in starts");
    let served = serve_the_stand_in(&mock).await;
    let channel = mock.config().channel_id;

    let asked = mock.say("demo: fix the parser");
    let thread_messages = format!("/api/v10/channels/{}/messages", asked.id);
    let journal = mock
        .wait_for(Duration::from_secs(30), |entries| {
            entries
                .iter()
                .any(|entry| matches!(entry, Entry::Rest { method, path, .. } if method == "POST" && *path == thread_messages))
                .then(|| entries.to_vec())
        })
        .await
        .expect("the daemon posts in the thread it made");

    // The gateway asks for the message's channel, to find its parent.
    let channel_path = format!("/api/v10/channels/{channel}");
    let looked_up = first(&journal, |entry| {
        matches!(entry, Entry::Rest { method, path, .. } if method == "GET" && *path == channel_path)
    })
    .expect("GET /channels/{channel}");

    // The thread hangs off the person's message, and has its id.
    let threads_path = format!("/api/v10/channels/{channel}/messages/{}/threads", asked.id);
    let made = first(&journal, |entry| {
        matches!(entry, Entry::Rest { method, path, .. } if method == "POST" && *path == threads_path)
    })
    .expect("POST .../threads");
    let Entry::Rest {
        status,
        request,
        response,
        ..
    } = &journal[made]
    else {
        unreachable!("found as a REST entry");
    };
    assert_eq!(*status, 201);
    assert_eq!(request["auto_archive_duration"], 1440);
    assert_eq!(response["id"], asked.id.to_string());

    let created = first(&journal, |entry| {
        entry.sent_event() == Some("THREAD_CREATE")
    })
    .expect("THREAD_CREATE");

    // The first post in the thread suppresses embeds and may ping users
    // only.
    let posted = first(&journal, |entry| {
        matches!(entry, Entry::Rest { method, path, .. } if method == "POST" && *path == thread_messages)
    })
    .expect("a post in the thread");
    let body = request_of(&journal[posted]);
    assert_eq!(body["flags"], 4, "{body}");
    assert_eq!(
        body["allowed_mentions"]["parse"],
        json!(["users"]),
        "{body}"
    );

    assert!(looked_up < made, "the lookup comes before the thread");
    assert!(created < posted, "THREAD_CREATE comes before the post");
    assert!(made < posted, "the thread is made before the post in it");

    served.daemon.shutdown().await;
    served.running.abort();
}

/// A slash command is claimed with a private deferral inside Discord's 3
/// seconds, then answered by editing the deferral in.
#[tokio::test]
async fn a_slash_command_is_deferred_privately_then_answered() {
    let mock = Mock::start(MockConfig::default())
        .await
        .expect("the stand-in starts");
    let served = serve_the_stand_in(&mock).await;
    register_commands(
        &served.http,
        serenity::model::id::GuildId::new(mock.config().guild_id),
        &silent(),
    )
    .await
    .expect("the commands register");
    let person = mock.config().people[0].id;
    mock.use_command(person, mock.config().channel_id, "status", json!([]))
        .expect("the person uses /status");

    let journal = mock
        .wait_for(Duration::from_secs(20), |entries| {
            entries
                .iter()
                .any(|entry| {
                    entry.is_call(
                        "PATCH",
                        "/api/v10/webhooks/{webhook_id}/{webhook_token}/messages/@original",
                    )
                })
                .then(|| entries.to_vec())
        })
        .await
        .expect("the answer is edited in");
    let callback = journal
        .iter()
        .find(|entry| {
            entry.is_call(
                "POST",
                "/api/v10/interactions/{interaction_id}/{interaction_token}/callback",
            )
        })
        .expect("a callback");
    let Entry::Rest {
        status, request, ..
    } = callback
    else {
        unreachable!("found as a REST entry");
    };
    assert_eq!(*status, 204, "claimed in time");
    assert_eq!(request["type"], 5);
    assert_eq!(request["data"]["flags"], 64);
    let edited = journal
        .iter()
        .rev()
        .find_map(|entry| match entry {
            Entry::Rest {
                method,
                status,
                request,
                ..
            } if method == "PATCH" && *status == 200 => Some(request.clone()),
            _ => None,
        })
        .expect("a successful edit");
    assert!(
        edited["content"]
            .as_str()
            .is_some_and(|text| !text.is_empty()),
        "{edited}"
    );

    served.daemon.shutdown().await;
    served.running.abort();
}
