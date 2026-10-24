//! What a test does that a person or a moderator does on Discord: write a
//! message, delete one, archive or delete a thread, use a command. Each goes
//! through the store like a REST write, so its events are dispatched the
//! same way.
//!
//! The same actions are served over HTTP under `/_mock/` for the binary.
//! Those routes are not Discord's and are not journalled.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::{Dispatch, NewMessage, Refusal, Store, Upload};
use crate::{Fault, MockHandle, Shared};

/// What an injected action made.
#[derive(Clone, Debug)]
pub struct Injected {
    /// The new message's or interaction's id.
    pub id: u64,
    /// The interaction token, for a command.
    pub token: Option<String>,
    /// The new message, as REST answers it; null for a command.
    pub value: Value,
}

impl Shared {
    fn act<T>(
        &self,
        change: impl FnOnce(&mut Store) -> Result<(T, Vec<Dispatch>), Refusal>,
    ) -> Result<T, Refusal> {
        let mut store = self.store.lock().expect("the store lock");
        let (answer, dispatches) = change(&mut store)?;
        self.gateway.broadcast(self, &store, &dispatches);
        Ok(answer)
    }

    fn post_as(
        &self,
        author_id: u64,
        channel_id: u64,
        content: &str,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<Injected, Refusal> {
        let uploads = files
            .into_iter()
            .map(|(filename, bytes)| Upload {
                content_type: Some(content_type_of(&filename).into()),
                filename,
                bytes,
            })
            .collect();
        let value = self.act(|store| {
            store.create_message(
                channel_id,
                author_id,
                NewMessage {
                    content: content.to_owned(),
                    uploads,
                    ..NewMessage::default()
                },
            )
        })?;
        let id = value["id"]
            .as_str()
            .and_then(|id| id.parse().ok())
            .unwrap_or_default();
        Ok(Injected {
            id,
            token: None,
            value,
        })
    }
}

/// A content type guessed from a file name, as a client sends one.
fn content_type_of(filename: &str) -> &'static str {
    match filename
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
    {
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "gif" => "image/gif",
        Some(ext) if ext == "txt" || ext == "md" => "text/plain; charset=utf-8",
        Some(ext) if ext == "json" => "application/json",
        _ => "application/octet-stream",
    }
}

impl MockHandle {
    /// A person writes `content`, with files, in a channel or a thread.
    ///
    /// # Errors
    ///
    /// Discord's refusal: an unknown channel, a locked archived thread.
    pub fn post_as(
        &self,
        author_id: u64,
        channel_id: u64,
        content: &str,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<Injected, Refusal> {
        self.shared.post_as(author_id, channel_id, content, files)
    }

    /// The first configured person writes `content` in the served channel.
    ///
    /// # Panics
    ///
    /// When the configuration names no person.
    pub fn say(&self, content: &str) -> Injected {
        let person = self.shared.config.people.first().expect("a person").id;
        self.post_as(person, self.shared.config.channel_id, content, Vec::new())
            .expect("the served channel takes a message")
    }

    /// Somebody deletes a message.
    ///
    /// # Errors
    ///
    /// Discord's refusal for an unknown channel or message.
    pub fn delete_message(&self, channel_id: u64, message_id: u64) -> Result<(), Refusal> {
        self.shared
            .act(|store| Ok(((), store.delete_message(channel_id, message_id)?)))
    }

    /// A moderator archives or unarchives a thread.
    ///
    /// # Errors
    ///
    /// Discord's refusal for an unknown channel or one that is no thread.
    pub fn set_archived(&self, thread_id: u64, archived: bool) -> Result<(), Refusal> {
        self.shared.act(|store| {
            let (_, events) = store.edit_thread(thread_id, Some(archived), None)?;
            Ok(((), events))
        })
    }

    /// A moderator deletes a thread.
    ///
    /// # Errors
    ///
    /// Discord's refusal for an unknown channel or one that is no thread.
    pub fn delete_thread(&self, thread_id: u64) -> Result<(), Refusal> {
        self.shared
            .act(|store| Ok(((), store.delete_thread(thread_id)?)))
    }

    /// A person uses a registered slash command in a channel or thread.
    ///
    /// # Errors
    ///
    /// When no command of that name is registered, or the channel is unknown.
    pub fn use_command(
        &self,
        user_id: u64,
        channel_id: u64,
        name: &str,
        options: Value,
    ) -> Result<Injected, Refusal> {
        let (id, token) = self.shared.act(|store| {
            let (id, token, events) = store.use_command(name, options, channel_id, user_id)?;
            Ok(((id, token), events))
        })?;
        Ok(Injected {
            id,
            token: Some(token),
            value: Value::Null,
        })
    }

    /// The thread or channel as REST answers it, for assertions.
    pub fn channel(&self, channel_id: u64) -> Option<Value> {
        let store = self.shared.store.lock().expect("the store lock");
        store
            .channel(channel_id)
            .ok()
            .map(|channel| store.channel_json(channel))
    }

    /// Ends the signature of every attachment URL issued so far.
    pub fn expire_attachments(&self) {
        let store = self.shared.store.lock().expect("the store lock");
        for attachment in store.attachments.values() {
            attachment.signature.borrow_mut().ex = 0;
        }
    }
}

/// A person's message, over HTTP.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PostBody {
    content: String,
    #[serde(default)]
    author_id: Option<u64>,
    #[serde(default)]
    channel_id: Option<u64>,
    /// Text files, by name.
    #[serde(default)]
    files: Vec<(String, String)>,
}

/// A fault, over HTTP.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, tag = "fault", rename_all = "snake_case")]
enum FaultBody {
    Drop,
    Reconnect,
    InvalidSession { resumable: bool },
    Close { code: u16 },
    WithholdAcks { withhold: bool },
    RequestHeartbeat,
}

fn refused(refusal: &Refusal) -> Response {
    (
        StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::BAD_REQUEST),
        Json(json!({ "message": refusal.message, "code": refusal.code })),
    )
        .into_response()
}

/// The `/_mock/` routes.
pub(crate) fn router(shared: Arc<Shared>) -> Router {
    Router::new()
        .route("/_mock/messages", post(post_message))
        .route("/_mock/fault", post(fault))
        .route("/_mock/journal", get(journal))
        .route("/_mock/gaps", get(gaps))
        .with_state(shared)
}

async fn post_message(State(shared): State<Arc<Shared>>, Json(body): Json<PostBody>) -> Response {
    let author = body
        .author_id
        .or_else(|| shared.config.people.first().map(|person| person.id))
        .unwrap_or_default();
    let channel = body.channel_id.unwrap_or(shared.config.channel_id);
    let files = body
        .files
        .into_iter()
        .map(|(name, text)| (name, text.into_bytes()))
        .collect();
    match shared.post_as(author, channel, &body.content, files) {
        Ok(injected) => Json(injected.value).into_response(),
        Err(refusal) => refused(&refusal),
    }
}

async fn fault(State(shared): State<Arc<Shared>>, Json(body): Json<FaultBody>) -> StatusCode {
    let fault = match body {
        FaultBody::Drop => Fault::Drop,
        FaultBody::Reconnect => Fault::Reconnect,
        FaultBody::InvalidSession { resumable } => Fault::InvalidSession { resumable },
        FaultBody::Close { code } => Fault::Close(code),
        FaultBody::WithholdAcks { withhold } => Fault::WithholdAcks(withhold),
        FaultBody::RequestHeartbeat => Fault::RequestHeartbeat,
    };
    shared.gateway.fault(&shared, fault);
    StatusCode::NO_CONTENT
}

async fn journal(State(shared): State<Arc<Shared>>) -> Json<Value> {
    Json(serde_json::to_value(shared.journal.entries()).unwrap_or(Value::Null))
}

async fn gaps(State(shared): State<Arc<Shared>>) -> Json<Value> {
    Json(json!(shared.journal.gaps()))
}
