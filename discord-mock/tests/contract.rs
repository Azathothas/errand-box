//! Test 4 of the plan, the Rust half: drive the fifteen REST operations
//! errand reaches, through serenity's own client, and keep each answer.
//!
//! With `DISCORD_MOCK_CORPUS` naming a directory, every answer under
//! `/api/v10` is written there as one JSON file, and
//! `scripts/discord-spec.sh --check` validates them against Discord's
//! OpenAPI spec. Without it, the test still drives every operation and
//! checks each status.

mod common;

use std::collections::BTreeSet;

use serde_json::json;
use serenity::builder::{
    Builder, CreateAttachment, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage,
    CreateThread, EditInteractionResponse, EditMessage, EditThread,
};
use serenity::model::channel::{AutoArchiveDuration, MessageFlags, ReactionType};
use serenity::model::id::{ChannelId, GuildId, InteractionId, MessageId};

use common::http;
use discord_mock::{Config, Entry, Mock};

/// The operations, as method and spec path.
const OPERATIONS: [(&str, &str); 15] = [
    ("GET", "/gateway"),
    ("GET", "/channels/{channel_id}"),
    ("PATCH", "/channels/{channel_id}"),
    ("POST", "/channels/{channel_id}/messages"),
    ("GET", "/channels/{channel_id}/messages/{message_id}"),
    ("PATCH", "/channels/{channel_id}/messages/{message_id}"),
    ("DELETE", "/channels/{channel_id}/messages/{message_id}"),
    (
        "PUT",
        "/channels/{channel_id}/messages/{message_id}/reactions/{emoji_name}/@me",
    ),
    (
        "DELETE",
        "/channels/{channel_id}/messages/{message_id}/reactions/{emoji_name}/@me",
    ),
    ("POST", "/channels/{channel_id}/typing"),
    (
        "POST",
        "/channels/{channel_id}/messages/{message_id}/threads",
    ),
    (
        "PUT",
        "/applications/{application_id}/guilds/{guild_id}/commands",
    ),
    (
        "POST",
        "/interactions/{interaction_id}/{interaction_token}/callback",
    ),
    (
        "PATCH",
        "/webhooks/{webhook_id}/{webhook_token}/messages/@original",
    ),
    ("POST", "/webhooks/{webhook_id}/{webhook_token}"),
];

/// The status a serenity error carries, for the refusals driven below.
fn status_of(error: &serenity::Error) -> Option<u16> {
    match error {
        serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(response)) => {
            Some(response.status_code.as_u16())
        }
        _ => None,
    }
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "one walk through every operation, in order"
)]
async fn every_operation_errand_reaches_answers_and_is_kept_for_the_spec() {
    let mock = Mock::start(Config::default())
        .await
        .expect("the mock starts");
    let http = http(&mock);
    http.set_application_id(mock.config().application_id.into());
    let channel = ChannelId::new(mock.config().channel_id);
    let guild = GuildId::new(mock.config().guild_id);
    let person = mock.config().people[0].id;

    http.get_gateway().await.expect("GET /gateway");
    channel.to_channel(&http).await.expect("the text channel");
    let asked = mock.say("demo: the contract");
    let thread = channel
        .create_thread_from_message(
            &http,
            MessageId::new(asked.id),
            CreateThread::new("contract").auto_archive_duration(AutoArchiveDuration::OneDay),
        )
        .await
        .expect("a thread");
    thread.id.to_channel(&http).await.expect("the thread");
    let posted = thread
        .id
        .send_message(
            &http,
            CreateMessage::new()
                .content("plain")
                .flags(MessageFlags::SUPPRESS_EMBEDS),
        )
        .await
        .expect("a JSON post");
    let uploaded = thread
        .id
        .send_message(
            &http,
            CreateMessage::new()
                .content("a file")
                .add_file(CreateAttachment::bytes(b"hello\n".to_vec(), "hello.txt")),
        )
        .await
        .expect("a multipart post");
    assert_eq!(uploaded.attachments.len(), 1);
    thread
        .id
        .message(&http, posted.id)
        .await
        .expect("GET a message");
    thread
        .id
        .edit_message(&http, posted.id, EditMessage::new().content("edited"))
        .await
        .expect("an edit");
    let check = ReactionType::Unicode("\u{2705}".into());
    thread
        .id
        .create_reaction(&http, posted.id, check.clone())
        .await
        .expect("a reaction");
    thread
        .id
        .delete_reaction(&http, posted.id, None, check)
        .await
        .expect("the reaction removed");
    http.broadcast_typing(thread.id).await.expect("typing");
    thread
        .id
        .delete_message(&http, uploaded.id)
        .await
        .expect("a delete");
    guild
        .set_commands(
            &http,
            vec![
                CreateCommand::new("status").description("what is running"),
                CreateCommand::new("stop").description("stop the session"),
            ],
        )
        .await
        .expect("commands");

    // A deferred, ephemeral answer, edited in and followed up, as errand
    // answers a slash command; then an immediate answer.
    let used = mock
        .use_command(person, thread.id.get(), "status", json!([]))
        .expect("a command");
    let token = used.token.clone().expect("a token");
    CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new().ephemeral(true))
        .execute(&http, (InteractionId::new(used.id), &token))
        .await
        .expect("a deferral");
    EditInteractionResponse::new()
        .content("nothing is running")
        .execute(&http, &token)
        .await
        .expect("the original edited in");
    CreateInteractionResponseFollowup::new()
        .content("and nothing is queued")
        .ephemeral(true)
        .execute(&http, (None, &token))
        .await
        .expect("a follow-up");
    let quick = mock
        .use_command(person, thread.id.get(), "stop", json!([]))
        .expect("a command");
    let quick_token = quick.token.clone().expect("a token");
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content("you are not permitted to use this bot"),
    )
    .execute(&http, (InteractionId::new(quick.id), &quick_token))
    .await
    .expect("an immediate answer");

    thread
        .id
        .edit_thread(&http, EditThread::new().archived(true))
        .await
        .expect("an archive");

    // Refusals Discord gives, so their bodies are checked too.
    let unknown = ChannelId::new(1)
        .to_channel(&http)
        .await
        .expect_err("no channel 1");
    assert_eq!(status_of(&unknown), Some(404));
    let twice = channel
        .create_thread_from_message(&http, MessageId::new(asked.id), CreateThread::new("again"))
        .await
        .expect_err("a second thread");
    assert_eq!(status_of(&twice), Some(400));
    let archived = thread
        .id
        .edit_message(&http, posted.id, EditMessage::new().content("late"))
        .await
        .expect_err("an edit in an archived thread");
    assert_eq!(status_of(&archived), Some(400));
    let again = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new())
        .execute(&http, (InteractionId::new(used.id), &token))
        .await
        .expect_err("a second answer");
    assert_eq!(status_of(&again), Some(400));

    let journal = mock.journal();
    let answered: BTreeSet<(String, String)> = journal
        .iter()
        .filter_map(|entry| match entry {
            Entry::Rest {
                method,
                route: Some(route),
                ..
            } => route
                .strip_prefix("/api/v10")
                .map(|route| (method.clone(), route.to_owned())),
            _ => None,
        })
        .collect();
    for (method, route) in OPERATIONS {
        assert!(
            answered.contains(&(method.to_owned(), route.to_owned())),
            "{method} {route} was driven"
        );
    }
    write_corpus(&journal);
}

/// Writes each `/api/v10` answer to `DISCORD_MOCK_CORPUS`, one file each.
fn write_corpus(journal: &[Entry]) {
    let Some(dir) = std::env::var_os("DISCORD_MOCK_CORPUS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("the corpus directory");
    let mut written = 0;
    for entry in journal {
        let Entry::Rest {
            method,
            route: Some(route),
            status,
            response,
            ..
        } = entry
        else {
            continue;
        };
        let Some(route) = route.strip_prefix("/api/v10") else {
            continue;
        };
        written += 1;
        let file = json!({
            "method": method,
            "route": route,
            "status": status,
            "body": response,
        });
        let path = dir.join(format!("{written:03}.json"));
        let text = serde_json::to_string_pretty(&file).expect("JSON");
        std::fs::write(path, text + "\n").expect("a corpus file");
    }
    assert!(
        written >= OPERATIONS.len(),
        "the corpus holds every operation"
    );
}
