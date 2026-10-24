//! The REST API errand reaches, the attachment CDN, and loud gaps.
//!
//! Routes are spelled as `discord/discord-api-spec` spells its paths, so a
//! journal entry's route is the spec's key, and `scripts/discord-spec.sh
//! --check` finds the schema an answer must match from it.
//!
//! A route the mock does not serve, a method it does not serve on a route,
//! and a body field it does not implement each answer 501 and are journalled
//! as a gap. A field serenity always sends with an inert value, `tts: false`
//! or `embeds: []`, is accepted while it keeps that value.

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::{Body, Bytes, to_bytes};
use axum::extract::{FromRequest, MatchedPath, Multipart, Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post, put};
use serde_json::{Map, Value, json};

use crate::Shared;
use crate::journal::Entry;
use crate::snowflake::now_ms;
use crate::state::{
    AUTO_ARCHIVE_DURATIONS, MessageEdit, NewMessage, Refusal, Upload, flag, refusal, same_secret,
};

/// The largest body the mock reads: Discord's 10 MiB upload limit for a bot
/// with room for the form around it.
const BODY_LIMIT: usize = 12 * 1024 * 1024;

/// Why a request was not served.
#[derive(Debug)]
pub(crate) enum ApiError {
    /// Discord's own answer.
    Discord(Refusal),
    /// The mock does not implement this: 501, and a gap in the journal.
    Gap(String),
}

impl From<Refusal> for ApiError {
    fn from(refusal: Refusal) -> Self {
        Self::Discord(refusal)
    }
}

/// Marks a response as a gap, for the journal.
#[derive(Clone, Debug)]
struct GapMarker(String);

/// A request body the journal shows in place of its raw bytes.
#[derive(Clone, Debug)]
struct Summary(Value);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Discord(refusal) => {
                let status =
                    StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::BAD_REQUEST);
                let body = json!({ "message": refusal.message, "code": refusal.code });
                (status, Json(body)).into_response()
            }
            Self::Gap(what) => {
                let body = json!({
                    "message": format!("discord-mock does not implement {what}"),
                    "code": 0,
                });
                let mut response = (StatusCode::NOT_IMPLEMENTED, Json(body)).into_response();
                response.extensions_mut().insert(GapMarker(what));
                response
            }
        }
    }
}

type Answer = Result<Response, ApiError>;

fn snowflake(text: &str) -> Result<u64, ApiError> {
    text.parse().map_err(|_| refusal::INVALID_FORM_BODY.into())
}

fn json_object(bytes: &[u8]) -> Result<Map<String, Value>, ApiError> {
    match serde_json::from_slice(bytes) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err(refusal::INVALID_FORM_BODY.into()),
    }
}

/// What a body field may hold when the mock does not implement it.
#[derive(Clone, Copy)]
enum Inert {
    False,
    Empty,
    Null,
}

impl Inert {
    fn holds(self, value: &Value) -> bool {
        match self {
            Self::False => matches!(value, Value::Bool(false) | Value::Null),
            Self::Empty => value.is_null() || value.as_array().is_some_and(Vec::is_empty),
            Self::Null => value.is_null(),
        }
    }
}

/// Refuses, as a gap, any field that is neither implemented nor inert.
fn check_fields(
    body: &Map<String, Value>,
    route: &str,
    implemented: &[&str],
    inert: &[(&str, Inert)],
) -> Result<(), ApiError> {
    for (key, value) in body {
        if implemented.contains(&key.as_str()) {
            continue;
        }
        let quiet = inert
            .iter()
            .any(|(name, kind)| name == key && kind.holds(value));
        if !quiet {
            return Err(ApiError::Gap(format!(
                "{route}: body field `{key}` = {value}"
            )));
        }
    }
    Ok(())
}

/// An optional field that must be a string when present.
fn text_field(body: &Map<String, Value>, key: &str) -> Result<Option<String>, ApiError> {
    match body.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(refusal::INVALID_FORM_BODY.into()),
    }
}

fn integer_field(body: &Map<String, Value>, key: &str) -> Result<Option<u64>, ApiError> {
    match body.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| refusal::INVALID_FORM_BODY.into()),
    }
}

fn bool_field(body: &Map<String, Value>, key: &str) -> Result<Option<bool>, ApiError> {
    match body.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(flag)) => Ok(Some(*flag)),
        Some(_) => Err(refusal::INVALID_FORM_BODY.into()),
    }
}

/// Checks `allowed_mentions` for the shape Discord takes; the mock does not
/// send notifications, so it has no other effect.
fn check_allowed_mentions(body: &Map<String, Value>) -> Result<(), ApiError> {
    let Some(mentions) = body
        .get("allowed_mentions")
        .filter(|value| !value.is_null())
    else {
        return Ok(());
    };
    let Value::Object(mentions) = mentions else {
        return Err(refusal::INVALID_FORM_BODY.into());
    };
    check_fields(
        mentions,
        "allowed_mentions",
        &["parse", "users", "roles", "replied_user"],
        &[],
    )?;
    let parse_ok = mentions.get("parse").is_none_or(|parse| {
        parse.as_array().is_some_and(|kinds| {
            kinds
                .iter()
                .all(|kind| matches!(kind.as_str(), Some("users" | "roles" | "everyone")))
        })
    });
    if !parse_ok {
        return Err(refusal::INVALID_FORM_BODY.into());
    }
    Ok(())
}

/// The flags a message may be created or edited with.
fn check_flags(flags: Option<u64>, allowed: u64, route: &str) -> Result<u64, ApiError> {
    let flags = flags.unwrap_or(0);
    if flags & !allowed != 0 {
        return Err(ApiError::Gap(format!("{route}: flags {flags}")));
    }
    Ok(flags)
}

impl Shared {
    /// Applies one change to the store and sends what it dispatches, under
    /// one hold of the store.
    fn mutate<T>(
        &self,
        change: impl FnOnce(
            &mut crate::state::Store,
        ) -> Result<(T, Vec<crate::state::Dispatch>), Refusal>,
    ) -> Result<T, ApiError> {
        let mut store = self.store.lock().expect("the store lock");
        let (answer, dispatches) = change(&mut store)?;
        self.gateway.broadcast(self, &store, &dispatches);
        Ok(answer)
    }
}

/// The router: Discord's routes under `/api/v10`, the CDN, and the test
/// control under `/_mock`.
pub(crate) fn router(shared: Arc<Shared>) -> Router {
    let api = Router::new()
        .route("/api/v10/gateway", get(gateway))
        .route(
            "/api/v10/channels/{channel_id}",
            get(get_channel).patch(update_channel),
        )
        .route(
            "/api/v10/channels/{channel_id}/messages",
            post(create_message),
        )
        .route(
            "/api/v10/channels/{channel_id}/messages/{message_id}",
            get(get_message)
                .patch(update_message)
                .delete(delete_message),
        )
        .route(
            "/api/v10/channels/{channel_id}/messages/{message_id}/reactions/{emoji_name}/@me",
            put(add_reaction).delete(remove_reaction),
        )
        .route("/api/v10/channels/{channel_id}/typing", post(typing))
        .route(
            "/api/v10/channels/{channel_id}/messages/{message_id}/threads",
            post(create_thread),
        )
        .route(
            "/api/v10/applications/{application_id}/guilds/{guild_id}/commands",
            put(put_commands),
        )
        .route(
            "/api/v10/interactions/{interaction_id}/{interaction_token}/callback",
            post(interaction_callback),
        )
        .route(
            "/api/v10/webhooks/{webhook_id}/{webhook_token}/messages/@original",
            patch(edit_original),
        )
        .route(
            "/api/v10/webhooks/{webhook_id}/{webhook_token}",
            post(follow_up),
        )
        .route(
            "/attachments/{channel_id}/{attachment_id}/{filename}",
            get(attachment),
        )
        .fallback(unimplemented)
        .method_not_allowed_fallback(unimplemented)
        .layer(middleware::from_fn_with_state(Arc::clone(&shared), record))
        .with_state(Arc::clone(&shared));
    api.merge(crate::control::router(shared))
        .layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
}

async fn unimplemented(method: Method, request: Request) -> ApiError {
    ApiError::Gap(format!("{method} {}", request.uri().path()))
}

/// Journals every call, checks the bot token, and adds the rate-limit
/// headers serenity's limiter reads (serenity 0.12.5
/// `src/http/ratelimiting.rs:337-359`; the limiter is off under the mock, so
/// no answer is ever 429).
async fn record(
    State(shared): State<Arc<Shared>>,
    matched: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_owned();
    let query = request.uri().query().map(str::to_owned);
    let route = matched.map(|matched| matched.as_str().to_owned());
    let is_api = path.starts_with("/api/");
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    // Interaction and webhook routes are authorized by the token in the
    // URL, as on Discord.
    let token_route =
        path.starts_with("/api/v10/interactions/") || path.starts_with("/api/v10/webhooks/");
    let authorized = !is_api
        || token_route
        || request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bot "))
            .is_some_and(|token| same_secret(token, &shared.config.token));
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, BODY_LIMIT).await.unwrap_or_default();
    let mut request_value = if bytes.is_empty() {
        Value::Null
    } else if content_type.starts_with("multipart/") {
        json!({ "multipart_bytes": bytes.len() })
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    let response = if authorized {
        next.run(Request::from_parts(parts, Body::from(bytes)))
            .await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "401: Unauthorized", "code": 0 })),
        )
            .into_response()
    };
    let (mut parts, body) = response.into_parts();
    let bytes = to_bytes(body, BODY_LIMIT).await.unwrap_or_default();
    if let Some(Summary(summary)) = parts.extensions.get::<Summary>() {
        request_value = summary.clone();
    }
    if let Some(GapMarker(what)) = parts.extensions.get::<GapMarker>() {
        shared.journal.push(Entry::Gap { what: what.clone() });
    }
    let response_value = if bytes.is_empty() {
        Value::Null
    } else if parts
        .headers
        .get(header::CONTENT_TYPE)
        .is_some_and(|value| value.as_bytes().starts_with(b"application/json"))
    {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    } else {
        json!({ "bytes": bytes.len() })
    };
    shared.journal.push(Entry::Rest {
        method,
        path,
        route,
        query,
        status: parts.status.as_u16(),
        request: request_value,
        response: response_value,
    });
    if is_api {
        let reset = now_ms() / 1000 + 1;
        for (name, value) in [
            ("x-ratelimit-limit", "5".to_owned()),
            ("x-ratelimit-remaining", "4".to_owned()),
            ("x-ratelimit-reset", format!("{reset}.000")),
            ("x-ratelimit-reset-after", "1.000".to_owned()),
            ("x-ratelimit-bucket", "discord-mock".to_owned()),
        ] {
            if let Ok(value) = HeaderValue::from_str(&value) {
                parts.headers.insert(name, value);
            }
        }
    }
    Response::from_parts(parts, Body::from(bytes))
}

async fn gateway(State(shared): State<Arc<Shared>>) -> Json<Value> {
    Json(json!({ "url": shared.ws_url }))
}

async fn get_channel(State(shared): State<Arc<Shared>>, Path(channel_id): Path<String>) -> Answer {
    let channel_id = snowflake(&channel_id)?;
    let store = shared.store.lock().expect("the store lock");
    let channel = store.channel(channel_id)?;
    Ok(Json(store.channel_json(channel)).into_response())
}

async fn update_channel(
    State(shared): State<Arc<Shared>>,
    Path(channel_id): Path<String>,
    body: Bytes,
) -> Answer {
    let channel_id = snowflake(&channel_id)?;
    let body = json_object(&body)?;
    check_fields(
        &body,
        "PATCH /channels/{channel_id}",
        &["archived", "locked"],
        &[],
    )?;
    let archived = bool_field(&body, "archived")?;
    let locked = bool_field(&body, "locked")?;
    let answer = shared.mutate(|store| store.edit_thread(channel_id, archived, locked))?;
    Ok(Json(answer).into_response())
}

/// The JSON payload and the files of a create-message request, and the
/// summary the journal shows for a multipart one.
async fn read_message_request(
    request: Request,
    route: &str,
) -> Result<(Map<String, Value>, Vec<(usize, Upload)>, Option<Value>), ApiError> {
    let multipart = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("multipart/form-data"));
    if !multipart {
        let bytes = Bytes::from_request(request, &())
            .await
            .map_err(|_| ApiError::Discord(refusal::INVALID_FORM_BODY))?;
        return Ok((json_object(&bytes)?, Vec::new(), None));
    }
    let mut form = Multipart::from_request(request, &())
        .await
        .map_err(|_| ApiError::Discord(refusal::INVALID_FORM_BODY))?;
    let mut payload = Map::new();
    let mut files = Vec::new();
    let mut parts = Vec::new();
    while let Some(field) = form
        .next_field()
        .await
        .map_err(|_| ApiError::Discord(refusal::INVALID_FORM_BODY))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        let filename = field.file_name().map(str::to_owned);
        let content_type = field.content_type().map(str::to_owned);
        let bytes = field
            .bytes()
            .await
            .map_err(|_| ApiError::Discord(refusal::INVALID_FORM_BODY))?;
        if name == "payload_json" {
            payload = json_object(&bytes)?;
            parts.push(json!({ "name": name, "json": Value::Object(payload.clone()) }));
            continue;
        }
        let index = name
            .strip_prefix("files[")
            .and_then(|rest| rest.strip_suffix(']'))
            .and_then(|index| index.parse::<usize>().ok())
            .ok_or_else(|| ApiError::Gap(format!("{route}: form part `{name}`")))?;
        parts.push(json!({
            "name": name,
            "filename": filename,
            "content_type": content_type,
            "size": bytes.len(),
        }));
        files.push((
            index,
            Upload {
                filename: filename.unwrap_or_else(|| format!("file{index}")),
                content_type,
                bytes: bytes.to_vec(),
            },
        ));
    }
    Ok((payload, files, Some(json!({ "multipart": parts }))))
}

/// Matches `attachments` entries to uploaded files by their placeholder id;
/// an entry's filename names the file.
fn uploads(
    body: &Map<String, Value>,
    mut files: Vec<(usize, Upload)>,
) -> Result<Vec<Upload>, ApiError> {
    if let Some(entries) = body.get("attachments").and_then(Value::as_array) {
        for entry in entries {
            let Value::Object(entry) = entry else {
                return Err(refusal::INVALID_FORM_BODY.into());
            };
            check_fields(
                entry,
                "attachments[]",
                &["id", "filename"],
                &[("description", Inert::Null)],
            )?;
            let index = entry
                .get("id")
                .and_then(|id| {
                    id.as_u64()
                        .or_else(|| id.as_str().and_then(|id| id.parse().ok()))
                })
                .and_then(|id| usize::try_from(id).ok())
                .ok_or(ApiError::Discord(refusal::INVALID_FORM_BODY))?;
            let file = files
                .iter_mut()
                .find(|(at, _)| *at == index)
                .ok_or(ApiError::Discord(refusal::INVALID_FORM_BODY))?;
            if let Some(name) = entry.get("filename").and_then(Value::as_str) {
                file.1.filename = name.to_owned();
            }
        }
    }
    files.sort_by_key(|(index, _)| *index);
    Ok(files.into_iter().map(|(_, upload)| upload).collect())
}

async fn create_message(
    State(shared): State<Arc<Shared>>,
    Path(channel_id): Path<String>,
    request: Request,
) -> Answer {
    const ROUTE: &str = "POST /channels/{channel_id}/messages";
    let channel_id = snowflake(&channel_id)?;
    let (body, files, summary) = read_message_request(request, ROUTE).await?;
    check_fields(
        &body,
        ROUTE,
        &[
            "content",
            "flags",
            "allowed_mentions",
            "message_reference",
            "attachments",
        ],
        &[
            ("tts", Inert::False),
            ("embeds", Inert::Empty),
            ("sticker_ids", Inert::Empty),
            ("components", Inert::Empty),
            ("enforce_nonce", Inert::False),
            ("nonce", Inert::Null),
        ],
    )?;
    let content = text_field(&body, "content")?.unwrap_or_default();
    // Only these two flags may be set on a new message by a bot that posts
    // no voice message and no components (docs `developers/resources/message.mdx`,
    // "Create Message", `flags`).
    let flags = check_flags(
        integer_field(&body, "flags")?,
        flag::SUPPRESS_EMBEDS | flag::SUPPRESS_NOTIFICATIONS,
        ROUTE,
    )?;
    check_allowed_mentions(&body)?;
    let reference = match body
        .get("message_reference")
        .filter(|value| !value.is_null())
    {
        None => None,
        Some(Value::Object(reference)) => {
            check_fields(
                reference,
                "message_reference",
                &[
                    "message_id",
                    "channel_id",
                    "guild_id",
                    "fail_if_not_exists",
                    "type",
                ],
                &[],
            )?;
            let id = reference
                .get("message_id")
                .and_then(Value::as_str)
                .ok_or(ApiError::Discord(refusal::INVALID_FORM_BODY))?;
            Some(snowflake(id)?)
        }
        Some(_) => return Err(refusal::INVALID_FORM_BODY.into()),
    };
    let uploads = uploads(&body, files)?;
    if content.is_empty() && uploads.is_empty() {
        // `developers/topics/opcodes-and-status-codes.mdx:267`
        return Err(ApiError::Discord(Refusal {
            status: 400,
            code: 50006,
            message: "Cannot send an empty message",
        }));
    }
    let bot = shared.config.bot_id;
    let answer = shared.mutate(|store| {
        store.create_message(
            channel_id,
            bot,
            NewMessage {
                content,
                flags,
                reference,
                uploads,
            },
        )
    })?;
    let mut response = Json(answer).into_response();
    if let Some(summary) = summary {
        response.extensions_mut().insert(Summary(summary));
    }
    Ok(response)
}

async fn get_message(
    State(shared): State<Arc<Shared>>,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> Answer {
    let (channel_id, message_id) = (snowflake(&channel_id)?, snowflake(&message_id)?);
    let store = shared.store.lock().expect("the store lock");
    Ok(Json(store.get_message(channel_id, message_id)?).into_response())
}

async fn update_message(
    State(shared): State<Arc<Shared>>,
    Path((channel_id, message_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "PATCH /channels/{channel_id}/messages/{message_id}";
    let (channel_id, message_id) = (snowflake(&channel_id)?, snowflake(&message_id)?);
    if headers
        .get(header::CONTENT_TYPE)
        .is_some_and(|value| value.as_bytes().starts_with(b"multipart/"))
    {
        return Err(ApiError::Gap(format!("{ROUTE}: a multipart edit")));
    }
    let body = json_object(&body)?;
    check_fields(
        &body,
        ROUTE,
        &["content", "flags", "allowed_mentions"],
        &[("embeds", Inert::Empty), ("components", Inert::Empty)],
    )?;
    check_allowed_mentions(&body)?;
    let flags = match integer_field(&body, "flags")? {
        None => None,
        // Of the flags an edit may set, the mock implements this one.
        flags => Some(check_flags(flags, flag::SUPPRESS_EMBEDS, ROUTE)?),
    };
    let edit = MessageEdit {
        content: text_field(&body, "content")?,
        flags,
    };
    let answer = shared.mutate(|store| store.edit_message(channel_id, message_id, edit))?;
    Ok(Json(answer).into_response())
}

async fn delete_message(
    State(shared): State<Arc<Shared>>,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> Answer {
    let (channel_id, message_id) = (snowflake(&channel_id)?, snowflake(&message_id)?);
    shared.mutate(|store| Ok(((), store.delete_message(channel_id, message_id)?)))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn react(shared: &Shared, path: (String, String, String), add: bool) -> Answer {
    let (channel_id, message_id, emoji) = path;
    let (channel_id, message_id) = (snowflake(&channel_id)?, snowflake(&message_id)?);
    let bot = shared.config.bot_id;
    shared.mutate(|store| Ok(((), store.react(channel_id, message_id, &emoji, bot, add)?)))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn add_reaction(
    State(shared): State<Arc<Shared>>,
    Path(path): Path<(String, String, String)>,
) -> Answer {
    react(&shared, path, true).await
}

async fn remove_reaction(
    State(shared): State<Arc<Shared>>,
    Path(path): Path<(String, String, String)>,
) -> Answer {
    react(&shared, path, false).await
}

async fn typing(State(shared): State<Arc<Shared>>, Path(channel_id): Path<String>) -> Answer {
    let channel_id = snowflake(&channel_id)?;
    let bot = shared.config.bot_id;
    shared.mutate(|store| Ok(((), store.typing(channel_id, bot)?)))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn create_thread(
    State(shared): State<Arc<Shared>>,
    Path((channel_id, message_id)): Path<(String, String)>,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "POST /channels/{channel_id}/messages/{message_id}/threads";
    let (channel_id, message_id) = (snowflake(&channel_id)?, snowflake(&message_id)?);
    let body = json_object(&body)?;
    check_fields(
        &body,
        ROUTE,
        &["name", "auto_archive_duration"],
        &[("rate_limit_per_user", Inert::Null)],
    )?;
    let name = text_field(&body, "name")?
        .filter(|name| (1..=100).contains(&name.chars().count()))
        .ok_or(ApiError::Discord(refusal::INVALID_FORM_BODY))?;
    let duration = integer_field(&body, "auto_archive_duration")?.unwrap_or(1440);
    if !AUTO_ARCHIVE_DURATIONS.contains(&duration) {
        return Err(refusal::INVALID_FORM_BODY.into());
    }
    let bot = shared.config.bot_id;
    let answer =
        shared.mutate(|store| store.create_thread(channel_id, message_id, name, duration, bot))?;
    Ok((StatusCode::CREATED, Json(answer)).into_response())
}

async fn put_commands(
    State(shared): State<Arc<Shared>>,
    Path((application_id, guild_id)): Path<(String, String)>,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "PUT /applications/{application_id}/guilds/{guild_id}/commands";
    if snowflake(&application_id)? != shared.config.application_id {
        // `developers/topics/opcodes-and-status-codes.mdx:262`
        return Err(ApiError::Discord(Refusal {
            status: 403,
            code: 50001,
            message: "Missing Access",
        }));
    }
    let guild_id = snowflake(&guild_id)?;
    if guild_id != shared.config.guild_id {
        // `developers/topics/opcodes-and-status-codes.mdx:144`
        return Err(ApiError::Discord(Refusal {
            status: 404,
            code: 10004,
            message: "Unknown Guild",
        }));
    }
    let Ok(Value::Array(commands)) = serde_json::from_slice::<Value>(&body) else {
        return Err(refusal::INVALID_FORM_BODY.into());
    };
    let mut wanted = Vec::new();
    for command in commands {
        let Value::Object(command) = command else {
            return Err(refusal::INVALID_FORM_BODY.into());
        };
        check_fields(
            &command,
            ROUTE,
            &[
                "name",
                "name_localizations",
                "description",
                "description_localizations",
                "options",
                "default_member_permissions",
                "dm_permission",
                "contexts",
                "integration_types",
                "type",
                "nsfw",
            ],
            &[("handler", Inert::Null), ("id", Inert::Null)],
        )?;
        if text_field(&command, "name")?.is_none() {
            return Err(refusal::INVALID_FORM_BODY.into());
        }
        wanted.push(command);
    }
    let answer = shared.mutate(|store| Ok((store.put_commands(guild_id, wanted), Vec::new())))?;
    Ok(Json(answer).into_response())
}

/// The fields of an interaction answer's message the mock implements.
const ANSWER_FIELDS: [&str; 3] = ["content", "flags", "allowed_mentions"];
const ANSWER_INERT: [(&str, Inert); 5] = [
    ("tts", Inert::False),
    ("embeds", Inert::Empty),
    ("components", Inert::Empty),
    ("attachments", Inert::Empty),
    ("poll", Inert::Null),
];
const ANSWER_FLAGS: u64 = flag::EPHEMERAL | flag::SUPPRESS_EMBEDS | flag::SUPPRESS_NOTIFICATIONS;

async fn interaction_callback(
    State(shared): State<Arc<Shared>>,
    Path((interaction_id, token)): Path<(String, String)>,
    Query(query): Query<Vec<(String, String)>>,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "POST /interactions/{interaction_id}/{interaction_token}/callback";
    let interaction_id = snowflake(&interaction_id)?;
    if let Some((key, value)) = query.first() {
        return Err(ApiError::Gap(format!("{ROUTE}: query {key}={value}")));
    }
    let body = json_object(&body)?;
    check_fields(&body, ROUTE, &["type", "data"], &[])?;
    let kind =
        integer_field(&body, "type")?.ok_or(ApiError::Discord(refusal::INVALID_FORM_BODY))?;
    if !matches!(kind, 4 | 5) {
        return Err(ApiError::Gap(format!("{ROUTE}: callback type {kind}")));
    }
    let data = match body.get("data") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(data)) => data.clone(),
        Some(_) => return Err(refusal::INVALID_FORM_BODY.into()),
    };
    check_fields(&data, ROUTE, &ANSWER_FIELDS, &ANSWER_INERT)?;
    check_allowed_mentions(&data)?;
    let flags = check_flags(integer_field(&data, "flags")?, ANSWER_FLAGS, ROUTE)?;
    let content = text_field(&data, "content")?;
    shared.mutate(|store| {
        Ok((
            (),
            store.answer_interaction(interaction_id, &token, kind, content, flags)?,
        ))
    })?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Checks a webhook URL's application id against the mock's.
fn check_application(shared: &Shared, webhook_id: &str) -> Result<(), ApiError> {
    if snowflake(webhook_id)? != shared.config.application_id {
        return Err(refusal::UNKNOWN_WEBHOOK.into());
    }
    Ok(())
}

async fn edit_original(
    State(shared): State<Arc<Shared>>,
    Path((webhook_id, token)): Path<(String, String)>,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "PATCH /webhooks/{webhook_id}/{webhook_token}/messages/@original";
    check_application(&shared, &webhook_id)?;
    let body = json_object(&body)?;
    check_fields(
        &body,
        ROUTE,
        &["content", "allowed_mentions"],
        &ANSWER_INERT,
    )?;
    check_allowed_mentions(&body)?;
    let content = text_field(&body, "content")?;
    let answer = shared.mutate(|store| store.edit_original(&token, content))?;
    Ok(Json(answer).into_response())
}

/// A follow-up message; `wait` is always true for an interaction's
/// follow-up (docs `developers/interactions/receiving-and-responding.mdx:477`).
async fn follow_up(
    State(shared): State<Arc<Shared>>,
    Path((webhook_id, token)): Path<(String, String)>,
    body: Bytes,
) -> Answer {
    const ROUTE: &str = "POST /webhooks/{webhook_id}/{webhook_token}";
    check_application(&shared, &webhook_id)?;
    let body = json_object(&body)?;
    check_fields(&body, ROUTE, &ANSWER_FIELDS, &ANSWER_INERT)?;
    check_allowed_mentions(&body)?;
    let flags = check_flags(integer_field(&body, "flags")?, ANSWER_FLAGS, ROUTE)?;
    let content = text_field(&body, "content")?.unwrap_or_default();
    let answer = shared.mutate(|store| store.follow_up(&token, content, flags))?;
    Ok(Json(answer).into_response())
}

/// An attachment's bytes, at the signed URL a message carries. A forged,
/// stale or expired signature answers 404, as Discord's CDN does
/// (docs `developers/reference.mdx:415-424`).
async fn attachment(
    State(shared): State<Arc<Shared>>,
    Path((channel_id, attachment_id, filename)): Path<(String, String, String)>,
    Query(query): Query<Vec<(String, String)>>,
) -> Response {
    let gone = || {
        (
            StatusCode::NOT_FOUND,
            "This content is no longer available.",
        )
            .into_response()
    };
    let value = |key: &str| {
        query
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    };
    let (Ok(channel_id), Ok(attachment_id)) =
        (channel_id.parse::<u64>(), attachment_id.parse::<u64>())
    else {
        return gone();
    };
    let store = shared.store.lock().expect("the store lock");
    let Some(file) = store.attachments.get(&attachment_id) else {
        return gone();
    };
    let signature = file.signature.borrow();
    let signed = value("ex") == Some(format!("{:x}", signature.ex).as_str())
        && value("is") == Some(format!("{:x}", signature.is).as_str())
        && value("hm").is_some_and(|hm| same_secret(hm, &signature.hm));
    if file.channel_id != channel_id
        || file.filename != filename
        || !signed
        || signature.ex <= now_ms() / 1000
    {
        return gone();
    }
    let content_type = file
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".into());
    ([(header::CONTENT_TYPE, content_type)], file.bytes.clone()).into_response()
}
