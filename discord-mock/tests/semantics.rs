//! The plan's implementation steps 1 to 4, each with Discord's rule it
//! codes, and the loud gaps. Raw requests go through reqwest, so a test can
//! send what serenity never would.

mod common;

use serde_json::{Value, json};
use serenity::builder::{CreateAttachment, CreateMessage, CreateThread, EditThread};
use serenity::model::channel::{Channel, ReactionType};
use serenity::model::id::{ChannelId, MessageId};

use common::{ERRAND_INTENTS, Ended, Socket, http};
use discord_mock::{Config, Entry, Mock, MockHandle};

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("a client")
}

/// A raw call as the bot; returns the status and the JSON body, or null.
async fn call(mock: &MockHandle, method: &str, path: &str, body: Option<Value>) -> (u16, Value) {
    let method = reqwest::Method::from_bytes(method.as_bytes()).expect("a method");
    let mut request = client()
        .request(method, format!("{}/api/v10{path}", mock.base_url))
        .header("Authorization", format!("Bot {}", mock.config().token));
    if let Some(body) = body {
        request = request
            .header("Content-Type", "application/json")
            .body(body.to_string());
    }
    let response = request.send().await.expect("an answer");
    let status = response.status().as_u16();
    let bytes = response.bytes().await.expect("a body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn status_of(error: &serenity::Error) -> Option<u16> {
    match error {
        serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(response)) => {
            Some(response.status_code.as_u16())
        }
        _ => None,
    }
}

fn code_of(error: &serenity::Error) -> Option<isize> {
    match error {
        serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(response)) => {
            Some(response.error.code)
        }
        _ => None,
    }
}

/// The dispatch names among the frames until `event`, in order.
async fn names_until(socket: &mut Socket, event: &str) -> Vec<String> {
    socket
        .until(event)
        .await
        .iter()
        .filter_map(|frame| frame.value["t"].as_str().map(str::to_owned))
        .collect()
}

// -- step 1: discovery and the handshake ----------------------------------

#[tokio::test]
async fn the_gateway_lookup_names_the_mocks_own_socket() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let (status, body) = call(&mock, "GET", "/gateway", None).await;
    assert_eq!(status, 200);
    assert_eq!(body, json!({ "url": mock.ws_url }));
    assert!(mock.ws_url.starts_with("ws://127.0.0.1:"));
}

#[tokio::test]
async fn hello_ready_guild_create_and_heartbeats_follow_discords_order() {
    let config = Config {
        heartbeat_interval_ms: 1000,
        ..Config::default()
    };
    let mock = Mock::start(config).await.expect("the mock starts");
    let mut socket = Socket::open(&mock).await;
    let hello = socket.identify(&mock.config().token, ERRAND_INTENTS).await;
    assert_eq!(hello.value["op"], 10);
    assert_eq!(hello.value["d"]["heartbeat_interval"], 1000);
    let ready = socket.frame().await;
    assert!(ready.compressed, "IDENTIFY asked for compression");
    let d = &ready.value["d"];
    assert_eq!(
        (ready.value["t"].clone(), ready.value["s"].clone()),
        (json!("READY"), json!(1))
    );
    assert_eq!(d["v"], 10);
    assert_eq!(d["resume_gateway_url"], mock.ws_url);
    assert_eq!(d["user"]["id"], mock.config().bot_id.to_string());
    assert_eq!(
        d["application"]["id"],
        mock.config().application_id.to_string()
    );
    assert_eq!(
        d["guilds"],
        json!([{ "id": mock.config().guild_id.to_string(), "unavailable": true }])
    );
    let guild = socket.frame().await;
    assert_eq!(
        (guild.value["t"].clone(), guild.value["s"].clone()),
        (json!("GUILD_CREATE"), json!(2))
    );
    socket.send(&json!({ "op": 1, "d": 2 })).await;
    let ack = socket.frame().await;
    assert_eq!(ack.value["op"], 11);
}

#[tokio::test]
async fn a_channel_and_a_thread_are_answered_by_get_channel() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let http = http(&mock);
    let channel = ChannelId::new(mock.config().channel_id);
    let Channel::Guild(text) = channel.to_channel(&http).await.expect("the channel") else {
        panic!("a guild channel");
    };
    assert_eq!(text.guild_id.get(), mock.config().guild_id);
    let asked = mock.say("demo: a thread");
    channel
        .create_thread_from_message(&http, MessageId::new(asked.id), CreateThread::new("t"))
        .await
        .expect("a thread");
    let Channel::Guild(thread) = ChannelId::new(asked.id)
        .to_channel(&http)
        .await
        .expect("the thread")
    else {
        panic!("a guild channel");
    };
    assert_eq!(thread.parent_id, Some(channel));
    assert!(thread.thread_metadata.is_some());
}

#[tokio::test]
async fn the_gateway_closes_as_discord_does_on_a_bad_handshake() {
    // `developers/topics/opcodes-and-status-codes.mdx:39-51`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let mut socket = Socket::open(&mock).await;
    socket.identify("mock.not-the-token", ERRAND_INTENTS).await;
    assert_eq!(socket.next().await.unwrap_err(), Ended::Closed(Some(4004)));

    let mut socket = Socket::open(&mock).await;
    socket.frame().await;
    socket.send(&json!({ "op": 99, "d": null })).await;
    assert_eq!(socket.next().await.unwrap_err(), Ended::Closed(Some(4001)));

    let closed = Mock::start(Config {
        message_content_intent: false,
        ..Config::default()
    })
    .await
    .expect("the mock starts");
    let mut socket = Socket::open(&closed).await;
    socket
        .identify(&closed.config().token, ERRAND_INTENTS)
        .await;
    assert_eq!(socket.next().await.unwrap_err(), Ended::Closed(Some(4014)));
}

#[tokio::test]
async fn too_many_sends_in_a_minute_close_with_4008() {
    let mock = Mock::start(Config {
        gateway_sends_per_minute: 3,
        ..Config::default()
    })
    .await
    .expect("the mock starts");
    let mut socket = Socket::open(&mock).await;
    socket.identify(&mock.config().token, ERRAND_INTENTS).await;
    socket.until("GUILD_CREATE").await;
    for _ in 0..3 {
        socket.send(&json!({ "op": 1, "d": 2 })).await;
    }
    loop {
        match socket.next().await {
            Ok(frame) => assert_eq!(frame.value["op"], 11),
            Err(ended) => {
                assert_eq!(ended, Ended::Closed(Some(4008)));
                break;
            }
        }
    }
}

#[tokio::test]
async fn without_the_content_intent_a_persons_message_arrives_empty() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let mut socket = Socket::open(&mock).await;
    socket.identify(&mock.config().token, 1 | 1 << 9).await;
    socket.until("GUILD_CREATE").await;
    mock.say("a secret plan");
    let created = socket.until("MESSAGE_CREATE").await;
    assert_eq!(created.last().expect("a frame").value["d"]["content"], "");
    let mention = format!("<@{}> please", mock.config().bot_id);
    mock.say(&mention);
    let created = socket.until("MESSAGE_CREATE").await;
    assert_eq!(
        created.last().expect("a frame").value["d"]["content"],
        mention
    );
}

// -- step 2: messages and threads --------------------------------------------

#[tokio::test]
async fn a_thread_takes_its_messages_id_and_a_second_is_refused() {
    // `developers/resources/channel.mdx:612-614`;
    // `developers/topics/opcodes-and-status-codes.mdx:344`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let http = http(&mock);
    let mut socket = Socket::open(&mock).await;
    socket.identify(&mock.config().token, ERRAND_INTENTS).await;
    socket.until("GUILD_CREATE").await;
    let channel = ChannelId::new(mock.config().channel_id);
    let asked = mock.say("demo: two threads");
    assert_eq!(
        names_until(&mut socket, "MESSAGE_CREATE").await,
        ["MESSAGE_CREATE"]
    );
    let thread = channel
        .create_thread_from_message(&http, MessageId::new(asked.id), CreateThread::new("one"))
        .await
        .expect("a thread");
    assert_eq!(thread.id.get(), asked.id);
    let events = socket.until("THREAD_MEMBER_UPDATE").await;
    let names: Vec<&str> = events
        .iter()
        .filter_map(|frame| frame.value["t"].as_str())
        .collect();
    assert_eq!(
        names,
        ["THREAD_CREATE", "MESSAGE_UPDATE", "THREAD_MEMBER_UPDATE"]
    );
    assert_eq!(events[0].value["d"]["newly_created"], true);
    assert_eq!(events[1].value["d"]["thread"]["id"], asked.id.to_string());

    let again = channel
        .create_thread_from_message(&http, MessageId::new(asked.id), CreateThread::new("two"))
        .await
        .expect_err("a second thread");
    assert_eq!(
        (status_of(&again), code_of(&again)),
        (Some(400), Some(160_004))
    );

    // The bot's own post comes back as MESSAGE_CREATE, as on Discord.
    thread.id.say(&http, "mine").await.expect("a post");
    let echoed = socket.until("MESSAGE_CREATE").await;
    let echoed = &echoed.last().expect("a frame").value["d"];
    assert_eq!(echoed["author"]["id"], mock.config().bot_id.to_string());
    assert_eq!(echoed["channel_id"], asked.id.to_string());
}

// -- step 3: edits, reactions, typing, archives ------------------------------

#[tokio::test]
async fn an_archived_thread_refuses_edits_and_reactions_and_a_message_unarchives_it() {
    // `developers/topics/threads.mdx:59`, `:198-203`;
    // `developers/topics/opcodes-and-status-codes.mdx:305`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let http = http(&mock);
    let mut socket = Socket::open(&mock).await;
    socket.identify(&mock.config().token, ERRAND_INTENTS).await;
    socket.until("GUILD_CREATE").await;
    let channel = ChannelId::new(mock.config().channel_id);
    let asked = mock.say("demo: archive");
    let thread = channel
        .create_thread_from_message(&http, MessageId::new(asked.id), CreateThread::new("a"))
        .await
        .expect("a thread");
    let posted = thread.id.say(&http, "before").await.expect("a post");
    let thumbs = ReactionType::Unicode("\u{1f44d}".into());
    thread
        .id
        .create_reaction(&http, posted.id, thumbs.clone())
        .await
        .expect("a reaction while open");
    http.broadcast_typing(thread.id)
        .await
        .expect("typing answers 204");
    socket.until("MESSAGE_CREATE").await;
    socket.until("MESSAGE_CREATE").await;

    thread
        .id
        .edit_thread(&http, EditThread::new().archived(true))
        .await
        .expect("an archive");
    assert_eq!(
        names_until(&mut socket, "THREAD_UPDATE").await,
        ["THREAD_UPDATE"]
    );
    let edit = thread
        .id
        .edit_message(
            &http,
            posted.id,
            serenity::builder::EditMessage::new().content("late"),
        )
        .await
        .expect_err("no edit in an archived thread");
    assert_eq!((status_of(&edit), code_of(&edit)), (Some(400), Some(50083)));
    let react = thread
        .id
        .delete_reaction(&http, posted.id, None, thumbs)
        .await
        .expect_err("no reaction in an archived thread");
    assert_eq!(code_of(&react), Some(50083));
    thread
        .id
        .delete_message(&http, posted.id)
        .await
        .expect("a delete is allowed in an archived thread");
    assert_eq!(
        names_until(&mut socket, "MESSAGE_DELETE").await,
        ["MESSAGE_DELETE"]
    );

    mock.post_as(
        mock.config().people[0].id,
        thread.id.get(),
        "still there?",
        Vec::new(),
    )
    .expect("a person's message");
    assert_eq!(
        names_until(&mut socket, "MESSAGE_CREATE").await,
        ["THREAD_UPDATE", "THREAD_MEMBER_UPDATE", "MESSAGE_CREATE"]
    );
    let open = mock.channel(thread.id.get()).expect("the thread");
    assert_eq!(open["thread_metadata"]["archived"], false);
}

#[tokio::test]
async fn only_the_bots_own_message_can_be_edited() {
    // `developers/topics/opcodes-and-status-codes.mdx:266`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let asked = mock.say("mine, not the bot's");
    let path = format!(
        "/channels/{}/messages/{}",
        mock.config().channel_id,
        asked.id
    );
    let (status, body) = call(&mock, "PATCH", &path, Some(json!({ "content": "x" }))).await;
    assert_eq!((status, body["code"].clone()), (403, json!(50005)));
    let (status, body) = call(&mock, "GET", &path, None).await;
    assert_eq!(
        (status, body["content"].clone()),
        (200, json!("mine, not the bot's"))
    );
}

// -- step 4: uploads and attachment URLs -------------------------------------

#[tokio::test]
async fn an_upload_is_served_back_at_a_signed_url_that_refuses_forgery_and_expiry() {
    // `developers/reference.mdx:415-424`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let http = http(&mock);
    let channel = ChannelId::new(mock.config().channel_id);
    let sent = channel
        .send_message(
            &http,
            CreateMessage::new()
                .content("a file")
                .add_file(CreateAttachment::bytes(
                    b"hello from errand\n".to_vec(),
                    "hello.txt",
                )),
        )
        .await
        .expect("an upload");
    let file = &sent.attachments[0];
    assert_eq!(file.filename, "hello.txt");
    assert_eq!(file.size, 18);
    assert!(
        file.url
            .starts_with(&format!("{}/attachments/", mock.base_url)),
        "{}",
        file.url
    );
    let fetched = client().get(&file.url).send().await.expect("the file");
    assert_eq!(fetched.status().as_u16(), 200);
    assert_eq!(
        fetched.bytes().await.expect("bytes").as_ref(),
        b"hello from errand\n"
    );

    let forged = file.url.replace("&hm=", "&hm=00");
    let refused = client().get(&forged).send().await.expect("an answer");
    assert_eq!(refused.status().as_u16(), 404);

    mock.expire_attachments();
    let expired = client().get(&file.url).send().await.expect("an answer");
    assert_eq!(expired.status().as_u16(), 404);
    // Fetching the message signs the URL afresh, as Discord does.
    let again = channel.message(&http, sent.id).await.expect("the message");
    assert_ne!(again.attachments[0].url, file.url);
    let fresh = client()
        .get(&again.attachments[0].url)
        .send()
        .await
        .expect("an answer");
    assert_eq!(fresh.status().as_u16(), 200);

    // A person's message carries its files the same way.
    let theirs = mock
        .post_as(
            mock.config().people[0].id,
            channel.get(),
            "see attached",
            vec![("notes.txt".into(), b"notes".to_vec())],
        )
        .expect("a person's upload");
    let url = theirs.value["attachments"][0]["url"]
        .as_str()
        .expect("a URL");
    let fetched = client().get(url).send().await.expect("the file");
    assert_eq!(fetched.bytes().await.expect("bytes").as_ref(), b"notes");
}

// -- loud gaps and authorization ---------------------------------------------

#[tokio::test]
async fn an_unimplemented_route_or_field_answers_501_and_is_a_gap() {
    let mut mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let (status, _) = call(&mock, "GET", "/users/@me", None).await;
    assert_eq!(status, 501);
    let path = format!("/channels/{}/messages", mock.config().channel_id);
    let (status, _) = call(&mock, "GET", &path, None).await;
    assert_eq!(status, 501, "a method the route does not serve");
    let (status, _) = call(
        &mock,
        "POST",
        &path,
        Some(json!({ "content": "x", "poll": { "question": { "text": "?" } } })),
    )
    .await;
    assert_eq!(status, 501, "a field the mock does not implement");
    let (status, _) = call(
        &mock,
        "POST",
        &path,
        Some(json!({ "content": "x", "tts": false })),
    )
    .await;
    assert_eq!(status, 200, "an inert field is accepted");
    let gaps = mock.take_gaps();
    assert_eq!(gaps.len(), 3, "{gaps:?}");
    assert!(gaps[0].contains("/api/v10/users/@me"));
    assert!(gaps[2].contains("`poll`"));
}

#[test]
fn a_gap_left_unread_fails_the_test_at_teardown() {
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    let mock = runtime.block_on(async {
        let mock = Mock::start(Config::default())
            .await
            .expect("the mock starts");
        call(&mock, "GET", "/users/@me", None).await;
        mock
    });
    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(mock)));
    assert!(dropped.is_err(), "dropping a handle with a gap panics");
}

#[tokio::test]
async fn a_call_without_the_bot_token_is_unauthorized() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let response = client()
        .get(format!("{}/api/v10/gateway", mock.base_url))
        .header("Authorization", "Bot mock.wrong")
        .send()
        .await
        .expect("an answer");
    assert_eq!(response.status().as_u16(), 401);
    let rest = mock
        .journal()
        .into_iter()
        .filter(|entry| matches!(entry, Entry::Rest { .. }))
        .count();
    assert_eq!(rest, 1, "the refusal is journalled");
}

// -- step 5: interactions ----------------------------------------------------

#[tokio::test]
async fn an_interaction_left_unanswered_for_three_seconds_is_gone() {
    // `developers/interactions/receiving-and-responding.mdx:351`
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let commands = format!(
        "/applications/{}/guilds/{}/commands",
        mock.config().application_id,
        mock.config().guild_id
    );
    let wanted = json!([{ "name": "status", "description": "s" }]);
    let (status, _) = call(&mock, "PUT", &commands, Some(wanted)).await;
    assert_eq!(status, 200);
    let person = mock.config().people[0].id;
    let late = mock
        .use_command(person, mock.config().channel_id, "status", json!([]))
        .expect("a command");
    tokio::time::sleep(std::time::Duration::from_millis(3200)).await;
    let callback = format!(
        "/interactions/{}/{}/callback",
        late.id,
        late.token.expect("a token")
    );
    let deferral = json!({ "type": 5, "data": { "flags": 64 } });
    let (status, body) = call(&mock, "POST", &callback, Some(deferral)).await;
    assert_eq!((status, body["code"].clone()), (404, json!(10062)));
}
