//! Test 5 of the plan: every dispatch errand can receive parses in two
//! clients written apart, in order, and a RESUME after a dropped socket
//! replays exactly what was missed, then RESUMED.
//!
//! serenity's `GatewayEvent` falls back to `Event::Unknown` for a payload
//! it cannot read (serenity 0.12.5 `src/model/event.rs:1425-1426`), and the
//! gateway logs and drops one that fails outright, so a wrong shape is
//! silent in a client. Here each frame must be a known event in serenity
//! and must deserialize in twilight-model.

mod common;

use std::collections::BTreeSet;

use serde_json::{Value, json};
use serenity::builder::{CreateCommand, CreateThread, EditMessage, EditThread};
use serenity::model::channel::{AutoArchiveDuration, ReactionType};
use serenity::model::event::{Event, GatewayEvent};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use twilight_model::gateway::event::GatewayEventDeserializer;

use common::{ERRAND_INTENTS, Frame, Socket, http};
use discord_mock::{Config, Fault, Mock};

/// The eleven dispatch types errand can receive with its intents.
const ERRAND_DISPATCHES: [&str; 11] = [
    "READY",
    "GUILD_CREATE",
    "MESSAGE_CREATE",
    "MESSAGE_UPDATE",
    "MESSAGE_DELETE",
    "THREAD_CREATE",
    "THREAD_UPDATE",
    "THREAD_DELETE",
    "THREAD_MEMBER_UPDATE",
    "INTERACTION_CREATE",
    "RESUMED",
];

/// Checks one dispatch frame in both clients; returns its event name.
fn parses_in_both(frame: &Frame) -> String {
    let event = frame.value["t"].as_str().expect("a dispatch").to_owned();
    match serde_json::from_str::<GatewayEvent>(&frame.text) {
        Ok(GatewayEvent::Dispatch(_, Event::Unknown(unknown))) => {
            panic!(
                "serenity read {event} as unknown: {unknown:?}\n{}",
                frame.text
            )
        }
        Ok(GatewayEvent::Dispatch(..)) => {}
        Ok(other) => panic!("serenity read {event} as {other:?}"),
        Err(error) => panic!("serenity refused {event}: {error}\n{}", frame.text),
    }
    let seed = GatewayEventDeserializer::from_json(&frame.text).expect("twilight finds op and t");
    let mut json = serde_json::Deserializer::from_str(&frame.text);
    if let Err(error) = serde::de::DeserializeSeed::deserialize(seed, &mut json) {
        panic!("twilight-model refused {event}: {error}\n{}", frame.text);
    }
    event
}

/// The dispatches among `frames`, checked in both clients and for a
/// sequence that rises by one from `after`.
fn dispatches(frames: &[Frame], after: &mut u64) -> Vec<String> {
    let mut names = Vec::new();
    for frame in frames.iter().filter(|frame| frame.value["op"] == 0) {
        let seq = frame.value["s"].as_u64().expect("a dispatch has s");
        assert_eq!(seq, *after + 1, "s rises by one: {}", frame.text);
        *after = seq;
        names.push(parses_in_both(frame));
    }
    names
}

#[tokio::test]
async fn every_dispatch_parses_in_two_clients_in_order_and_a_resume_replays_the_gap() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let token = mock.config().token.clone();
    let channel = ChannelId::new(mock.config().channel_id);
    let guild = GuildId::new(mock.config().guild_id);
    let person = mock.config().people[0].id;
    let http = http(&mock);
    http.set_application_id(mock.config().application_id.into());

    let mut socket = Socket::open(&mock).await;
    let hello = socket.identify(&token, ERRAND_INTENTS).await;
    assert_eq!(hello.value["op"], 10);
    let mut seq = 0;
    let mut seen = Vec::new();
    let ready = socket.until("GUILD_CREATE").await;
    let session_id = ready[0].value["d"]["session_id"]
        .as_str()
        .expect("READY carries the session")
        .to_owned();
    seen.extend(dispatches(&ready, &mut seq));

    // A person writes; the bot opens a thread, posts, edits, reacts,
    // deletes, and archives it.
    let asked = mock.say("demo: one thing");
    seen.extend(dispatches(&socket.until("MESSAGE_CREATE").await, &mut seq));
    let thread = channel
        .create_thread_from_message(
            &http,
            MessageId::new(asked.id),
            CreateThread::new("demo").auto_archive_duration(AutoArchiveDuration::OneDay),
        )
        .await
        .expect("a thread");
    seen.extend(dispatches(
        &socket.until("THREAD_MEMBER_UPDATE").await,
        &mut seq,
    ));
    let posted = thread
        .id
        .say(&http, "working")
        .await
        .expect("a post in the thread");
    seen.extend(dispatches(&socket.until("MESSAGE_CREATE").await, &mut seq));
    thread
        .id
        .edit_message(
            &http,
            posted.id,
            EditMessage::new().content("working on it"),
        )
        .await
        .expect("an edit");
    seen.extend(dispatches(&socket.until("MESSAGE_UPDATE").await, &mut seq));
    // Neither a reaction nor typing reaches a session without their intents.
    thread
        .id
        .create_reaction(&http, posted.id, ReactionType::Unicode("\u{2705}".into()))
        .await
        .expect("a reaction");
    http.broadcast_typing(thread.id).await.expect("typing");
    thread
        .id
        .delete_message(&http, posted.id)
        .await
        .expect("a delete");
    let deleted = socket.until("MESSAGE_DELETE").await;
    assert_eq!(deleted.len(), 1, "nothing between the delete and its event");
    seen.extend(dispatches(&deleted, &mut seq));
    guild
        .set_commands(
            &http,
            vec![CreateCommand::new("status").description("status")],
        )
        .await
        .expect("commands");
    mock.use_command(person, thread.id.get(), "status", json!([]))
        .expect("a command");
    seen.extend(dispatches(
        &socket.until("INTERACTION_CREATE").await,
        &mut seq,
    ));
    thread
        .id
        .edit_thread(&http, EditThread::new().archived(true))
        .await
        .expect("an archive");
    seen.extend(dispatches(&socket.until("THREAD_UPDATE").await, &mut seq));

    // The socket drops; two events happen while it is down.
    mock.fault(Fault::Drop);
    assert!(socket.next().await.is_err(), "the socket ends");
    mock.say("said while the bot was away");
    mock.delete_thread(thread.id.get()).expect("a delete");

    let mut resumed = Socket::open(&mock).await;
    assert_eq!(resumed.frame().await.value["op"], 10);
    resumed
        .send(&json!({
            "op": 6,
            "d": { "token": format!("Bot {token}"), "session_id": session_id, "seq": seq },
        }))
        .await;
    let replayed = resumed.until("RESUMED").await;
    let names = dispatches(&replayed, &mut seq);
    assert_eq!(names, ["MESSAGE_CREATE", "THREAD_DELETE", "RESUMED"]);
    seen.extend(names);

    let seen: BTreeSet<&str> = seen.iter().map(String::as_str).collect();
    let errand: BTreeSet<&str> = ERRAND_DISPATCHES.into_iter().collect();
    assert_eq!(
        seen, errand,
        "every dispatch errand can receive, and no other"
    );

    let journal_events: Vec<Value> = mock
        .journal()
        .iter()
        .filter_map(|entry| entry.sent_event().map(|event| json!(event)))
        .collect();
    assert!(
        !journal_events.contains(&json!("MESSAGE_REACTION_ADD"))
            && !journal_events.contains(&json!("TYPING_START")),
        "no event of an intent errand does not ask for"
    );
}

#[tokio::test]
async fn a_resume_of_an_unknown_session_is_refused_with_op_9() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let mut socket = Socket::open(&mock).await;
    socket.frame().await;
    socket
        .send(&json!({
            "op": 6,
            "d": { "token": format!("Bot {}", mock.config().token), "session_id": "gone", "seq": 3 },
        }))
        .await;
    let refused = socket.frame().await;
    assert_eq!(refused.value["op"], 9);
    assert_eq!(refused.value["d"], false);
}
