//! The gateway: one socket per connection, sessions that outlive a socket,
//! sequence numbers, replay on RESUME, intents, and scripted faults.
//!
//! A session is made by IDENTIFY and keeps every dispatch it was sent, so a
//! RESUME on a new socket replays exactly what came after the client's last
//! `seq`, then RESUMED. A dispatch reaches a session only when its intents
//! ask for it (`discord/discord-api-docs@6128d99116af
//! developers/events/gateway.mdx:313-392`).
//!
//! Frames are journalled when they are queued for a socket, so a dispatch a
//! REST call causes is in the journal before that call's answer.

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use flate2::Compression;
use flate2::write::ZlibEncoder;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

use crate::Shared;
use crate::journal::Entry;
use crate::state::{Dispatch, Store, intent, random_hex, same_secret};

/// Every intent bit Discord defines, through `1 << 25`
/// (`developers/events/gateway.mdx:313-430`); any other bit closes with 4013.
const KNOWN_INTENTS: u64 = (1 << 26) - 1;

/// The application flag that lets an app ask for the content intent:
/// `GATEWAY_MESSAGE_CONTENT_LIMITED`, as an unverified app has it.
const APP_CONTENT_FLAG: u64 = 1 << 19;

/// A fault a test asks the gateway for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// End every socket without a close frame, as a network drop does; the
    /// sessions stay, so a client resumes.
    Drop,
    /// Send op 7, Reconnect.
    Reconnect,
    /// Send op 9, Invalid Session; when not resumable, the sessions end.
    InvalidSession { resumable: bool },
    /// Close every socket with this code.
    Close(u16),
    /// Stop, or start again, answering heartbeats with op 11.
    WithholdAcks(bool),
    /// Send op 1, asking the client for a heartbeat now.
    RequestHeartbeat,
}

/// What a connection's task writes next.
#[derive(Debug)]
pub(crate) enum Outbound {
    Frame { payload: Value, compress: bool },
    Close(u16, String),
    Drop,
}

/// A session, made by IDENTIFY.
#[derive(Debug)]
struct Session {
    intents: u64,
    compress: bool,
    seq: u64,
    /// Every dispatch sent, by sequence number, for a RESUME to replay.
    sent: Vec<(u64, Value)>,
    /// The socket it is on now, if any.
    connection: Option<u64>,
}

#[derive(Debug, Default)]
struct Hub {
    sessions: HashMap<String, Session>,
    connections: HashMap<u64, mpsc::UnboundedSender<Outbound>>,
    next_connection: u64,
    withhold_acks: bool,
}

/// The gateway's state, shared by every socket.
#[derive(Debug, Default)]
pub(crate) struct Gateway {
    hub: Mutex<Hub>,
}

/// A frame with no sequence number: HELLO, ACK, RECONNECT, INVALID_SESSION.
fn control_frame(op: u64, d: Value) -> Value {
    json!({ "t": null, "s": null, "op": op, "d": d })
}

impl Gateway {
    /// Queues a frame on a connection, journalling it.
    fn queue(
        shared: &Shared,
        sender: &mpsc::UnboundedSender<Outbound>,
        payload: Value,
        compress: bool,
    ) {
        shared.journal.push(Entry::Sent {
            payload: payload.clone(),
            compressed: compress,
        });
        // A send fails only when the socket's task has ended; the frame is
        // then for a socket that is gone, which is what a drop is.
        let _ = sender.send(Outbound::Frame { payload, compress });
    }

    /// Sends each dispatch to every session whose intents ask for it.
    ///
    /// Called while the caller holds the store, so dispatches leave in the
    /// order the store changed.
    pub(crate) fn broadcast(&self, shared: &Shared, store: &Store, dispatches: &[Dispatch]) {
        let mut hub = self.hub.lock().expect("the gateway lock");
        let Hub {
            sessions,
            connections,
            ..
        } = &mut *hub;
        for dispatch in dispatches {
            for session in sessions.values_mut() {
                if dispatch.intent != intent::NONE && session.intents & dispatch.intent == 0 {
                    continue;
                }
                let data = shaped(store, session.intents, dispatch);
                session.seq += 1;
                let payload = json!({ "t": dispatch.event, "s": session.seq, "op": 0, "d": data });
                session.sent.push((session.seq, payload.clone()));
                if let Some(sender) = session.connection.and_then(|id| connections.get(&id)) {
                    let compress = session.compress && shared.config.compress;
                    Self::queue(shared, sender, payload, compress);
                }
            }
        }
    }

    /// Applies a fault to every open socket.
    pub(crate) fn fault(&self, shared: &Shared, fault: Fault) {
        let mut hub = self.hub.lock().expect("the gateway lock");
        match fault {
            Fault::WithholdAcks(withhold) => hub.withhold_acks = withhold,
            Fault::Drop => {
                for sender in hub.connections.values() {
                    let _ = sender.send(Outbound::Drop);
                }
            }
            Fault::Close(code) => {
                for sender in hub.connections.values() {
                    let _ = sender.send(Outbound::Close(code, "closed by a test".into()));
                }
            }
            Fault::Reconnect => {
                for sender in hub.connections.values() {
                    Self::queue(shared, sender, control_frame(7, Value::Null), false);
                }
            }
            Fault::RequestHeartbeat => {
                for sender in hub.connections.values() {
                    Self::queue(shared, sender, control_frame(1, Value::Null), false);
                }
            }
            Fault::InvalidSession { resumable } => {
                for sender in hub.connections.values() {
                    Self::queue(
                        shared,
                        sender,
                        control_frame(9, Value::Bool(resumable)),
                        false,
                    );
                }
                if !resumable {
                    hub.sessions.clear();
                }
            }
        }
    }

    fn attach(&self, sender: mpsc::UnboundedSender<Outbound>) -> u64 {
        let mut hub = self.hub.lock().expect("the gateway lock");
        hub.next_connection += 1;
        let id = hub.next_connection;
        hub.connections.insert(id, sender);
        id
    }

    fn detach(&self, connection: u64) {
        let mut hub = self.hub.lock().expect("the gateway lock");
        hub.connections.remove(&connection);
        for session in hub.sessions.values_mut() {
            if session.connection == Some(connection) {
                session.connection = None;
            }
        }
    }
}

/// A dispatch's data as one session sees it. Without the content intent, a
/// message neither from the bot nor mentioning it arrives with its content
/// fields empty (`developers/events/gateway.mdx:497-503`).
fn shaped(store: &Store, intents: u64, dispatch: &Dispatch) -> Value {
    let mut data = dispatch.data.clone();
    let is_message = matches!(dispatch.event, "MESSAGE_CREATE" | "MESSAGE_UPDATE");
    if !is_message || intents & intent::MESSAGE_CONTENT != 0 {
        return data;
    }
    let bot = store.config.bot_id.to_string();
    let from_bot = data["author"]["id"].as_str() == Some(bot.as_str());
    let mentions_bot = data["mentions"].as_array().is_some_and(|users| {
        users
            .iter()
            .any(|user| user["id"].as_str() == Some(bot.as_str()))
    });
    if !from_bot && !mentions_bot {
        data["content"] = Value::String(String::new());
        data["embeds"] = json!([]);
        data["attachments"] = json!([]);
        data["components"] = json!([]);
    }
    data
}

/// Accepts gateway sockets until the task is aborted.
pub(crate) async fn serve(shared: Arc<Shared>, listener: TcpListener) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        tokio::spawn(connection(Arc::clone(&shared), stream));
    }
}

/// Why a socket ends: a close code and its reason, or a drop.
type Ending = Option<(u16, String)>;

/// One socket, from the handshake to its end.
async fn connection(shared: Arc<Shared>, stream: TcpStream) {
    let asked = Arc::new(Mutex::new(String::new()));
    let seen = Arc::clone(&asked);
    #[allow(
        clippy::result_large_err,
        reason = "tungstenite's handshake callback fixes the error type"
    )]
    let handshake = move |request: &Request, response: Response| {
        *seen.lock().expect("the query lock") =
            request.uri().query().unwrap_or_default().to_owned();
        Ok(response)
    };
    let Ok(socket) = tokio_tungstenite::accept_hdr_async(stream, handshake).await else {
        return;
    };
    let query = asked.lock().expect("the query lock").clone();
    shared.journal.push(Entry::Connect {
        query: query.clone(),
    });
    let (mut sink, mut source) = socket.split();
    let (sender, mut outbound) = mpsc::unbounded_channel();
    let id = shared.gateway.attach(sender.clone());
    let mut link = Link {
        connection: id,
        sender,
        session: None,
        recent: VecDeque::new(),
    };

    let mut ending: Ending = if query.split('&').any(|pair| pair == "v=10") {
        let hello = json!({
            "heartbeat_interval": shared.config.heartbeat_interval_ms,
            "_trace": ["[\"discord-mock\"]"],
        });
        Gateway::queue(&shared, &link.sender, control_frame(10, hello), false);
        None
    } else {
        // `developers/topics/opcodes-and-status-codes.mdx:49`
        Some((4012, "Invalid API version".into()))
    };
    let mut dropped = false;
    while ending.is_none() {
        tokio::select! {
            incoming = source.next() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    if let Err(close) = link.receive(&shared, &text) {
                        ending = Some(close);
                    }
                }
                // The client cannot send compressed or binary frames
                // (`developers/events/gateway.mdx:542`).
                Some(Ok(Message::Binary(_))) => {
                    ending = Some((4002, "Error while decoding payload.".into()));
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            next = outbound.recv() => match next {
                Some(Outbound::Frame { payload, compress }) => {
                    if sink.send(encode(&payload, compress)).await.is_err() {
                        break;
                    }
                }
                Some(Outbound::Close(code, reason)) => ending = Some((code, reason)),
                Some(Outbound::Drop) | None => {
                    dropped = true;
                    break;
                }
            },
        }
    }
    shared.gateway.detach(id);
    if dropped {
        shared.journal.push(Entry::Closed {
            code: None,
            reason: "dropped".into(),
        });
        return;
    }
    if let Some((code, reason)) = ending {
        // Frames queued before the close still go out first.
        while let Ok(Outbound::Frame { payload, compress }) = outbound.try_recv() {
            let _ = sink.send(encode(&payload, compress)).await;
        }
        shared.journal.push(Entry::Closed {
            code: Some(code),
            reason: reason.clone(),
        });
        let frame = CloseFrame {
            code: CloseCode::from(code),
            reason: reason.into(),
        };
        let _ = sink.send(Message::Close(Some(frame))).await;
        let _ = sink.close().await;
    }
}

/// A frame on the wire: zlib-compressed as a binary frame when the session
/// asked for payload compression (`developers/events/gateway.mdx:529-535`).
fn encode(payload: &Value, compress: bool) -> Message {
    let text = payload.to_string();
    if !compress {
        return Message::Text(text);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .expect("writing to memory");
    Message::Binary(encoder.finish().expect("writing to memory"))
}

/// One socket's side of the protocol.
struct Link {
    connection: u64,
    sender: mpsc::UnboundedSender<Outbound>,
    session: Option<String>,
    /// When each recent client frame came, for the send limit.
    recent: VecDeque<Instant>,
}

/// How long the send limit counts over (`developers/events/gateway.mdx:509`).
const SEND_WINDOW: Duration = Duration::from_secs(60);

impl Link {
    /// Handles one client frame; an error is the close it earns.
    fn receive(&mut self, shared: &Shared, text: &str) -> Result<(), (u16, String)> {
        let now = Instant::now();
        while self
            .recent
            .front()
            .is_some_and(|at| now.duration_since(*at) > SEND_WINDOW)
        {
            self.recent.pop_front();
        }
        self.recent.push_back(now);
        let Ok(frame) = serde_json::from_str::<Value>(text) else {
            return Err((4002, "Error while decoding payload.".into()));
        };
        let op = frame["op"].as_u64();
        shared.journal.push(Entry::Received {
            op: op.unwrap_or(u64::MAX),
            payload: frame.clone(),
        });
        if self.recent.len() > shared.config.gateway_sends_per_minute {
            // `developers/topics/opcodes-and-status-codes.mdx:45`
            return Err((4008, "Rate limited.".into()));
        }
        match op {
            Some(1) => {
                let hub = shared.gateway.hub.lock().expect("the gateway lock");
                if !hub.withhold_acks {
                    Gateway::queue(shared, &self.sender, control_frame(11, Value::Null), false);
                }
                Ok(())
            }
            Some(2) => self.identify(shared, &frame["d"]),
            Some(3) => {
                if self.session.is_none() {
                    return Err((4003, "Not authenticated.".into()));
                }
                Ok(())
            }
            Some(6) => self.resume(shared, &frame["d"]),
            Some(other @ (4 | 8 | 31 | 36 | 37)) => {
                shared.journal.push(Entry::Gap {
                    what: format!("gateway op {other}"),
                });
                Ok(())
            }
            _ => Err((4001, "Unknown opcode.".into())),
        }
    }

    /// Whether `token`, with or without the `Bot ` serenity puts in front of
    /// it, is the bot's.
    fn token_matches(shared: &Shared, token: &Value) -> bool {
        let token = token.as_str().unwrap_or_default();
        let token = token.strip_prefix("Bot ").unwrap_or(token);
        same_secret(token, &shared.config.token)
    }

    fn identify(&mut self, shared: &Shared, d: &Value) -> Result<(), (u16, String)> {
        // Close codes: `developers/topics/opcodes-and-status-codes.mdx:42-51`.
        if self.session.is_some() {
            return Err((4005, "Already authenticated.".into()));
        }
        if !Self::token_matches(shared, &d["token"]) {
            return Err((4004, "Authentication failed.".into()));
        }
        let Some(intents) = d["intents"]
            .as_u64()
            .filter(|bits| bits & !KNOWN_INTENTS == 0)
        else {
            return Err((4013, "Invalid intent(s).".into()));
        };
        if intents & intent::MESSAGE_CONTENT != 0 && !shared.config.message_content_intent {
            return Err((4014, "Disallowed intent(s).".into()));
        }
        let shard = d.get("shard").cloned();
        if shard.as_ref().is_some_and(|shard| shard != &json!([0, 1])) {
            return Err((4010, "Invalid shard.".into()));
        }
        let session_id = random_hex(16);
        let store = shared.store.lock().expect("the store lock");
        let mut user = store.user_json(store.config.bot_id);
        user["verified"] = Value::Bool(true);
        user["mfa_enabled"] = Value::Bool(false);
        user["email"] = Value::Null;
        let application_flags = if shared.config.message_content_intent {
            APP_CONTENT_FLAG
        } else {
            0
        };
        let mut ready = json!({
            "v": 10,
            "user_settings": {},
            "user": user,
            "session_type": "normal",
            "session_id": session_id,
            "resume_gateway_url": shared.ws_url,
            "relationships": [],
            "private_channels": [],
            "presences": [],
            "guilds": [{ "id": store.config.guild_id.to_string(), "unavailable": true }],
            "guild_join_requests": [],
            "geo_ordered_rtc_regions": [],
            "auth": {},
            "application": {
                "id": store.config.application_id.to_string(),
                "flags": application_flags,
            },
            "_trace": ["[\"discord-mock\"]"],
        });
        if let Some(shard) = shard {
            ready["shard"] = shard;
        }
        let guild = store.guild_json();
        {
            let mut hub = shared.gateway.hub.lock().expect("the gateway lock");
            hub.sessions.insert(
                session_id.clone(),
                Session {
                    intents,
                    compress: d["compress"].as_bool().unwrap_or(false),
                    seq: 0,
                    sent: Vec::new(),
                    connection: Some(self.connection),
                },
            );
        }
        self.session = Some(session_id);
        // GUILD_CREATE follows READY for each guild once it is available.
        // Sent through the one dispatch path, to this session only, so `s`
        // is 1 and then 2.
        self.dispatch_here(shared, "READY", ready);
        self.dispatch_here(shared, "GUILD_CREATE", guild);
        drop(store);
        Ok(())
    }

    /// Sends one dispatch to this socket's session alone.
    fn dispatch_here(&self, shared: &Shared, event: &str, data: Value) {
        let Some(session_id) = &self.session else {
            return;
        };
        let mut hub = shared.gateway.hub.lock().expect("the gateway lock");
        let Some(session) = hub.sessions.get_mut(session_id) else {
            return;
        };
        session.seq += 1;
        let payload = json!({ "t": event, "s": session.seq, "op": 0, "d": data });
        session.sent.push((session.seq, payload.clone()));
        let compress = session.compress && shared.config.compress;
        Gateway::queue(shared, &self.sender, payload, compress);
    }

    fn resume(&mut self, shared: &Shared, d: &Value) -> Result<(), (u16, String)> {
        if self.session.is_some() {
            return Err((4005, "Already authenticated.".into()));
        }
        if !Self::token_matches(shared, &d["token"]) {
            return Err((4004, "Authentication failed.".into()));
        }
        let session_id = d["session_id"].as_str().unwrap_or_default().to_owned();
        let after = d["seq"].as_u64().unwrap_or(0);
        // The store first, as everywhere, so no dispatch slips between the
        // replay and RESUMED.
        let _store = shared.store.lock().expect("the store lock");
        let mut hub = shared.gateway.hub.lock().expect("the gateway lock");
        let Some(session) = hub.sessions.get_mut(&session_id) else {
            Gateway::queue(
                shared,
                &self.sender,
                control_frame(9, Value::Bool(false)),
                false,
            );
            return Ok(());
        };
        if after > session.seq {
            return Err((4007, "Invalid seq.".into()));
        }
        session.connection = Some(self.connection);
        let compress = session.compress && shared.config.compress;
        for (_, payload) in session.sent.iter().filter(|(seq, _)| *seq > after) {
            Gateway::queue(shared, &self.sender, payload.clone(), compress);
        }
        session.seq += 1;
        let payload = json!({
            "t": "RESUMED",
            "s": session.seq,
            "op": 0,
            "d": { "_trace": ["[\"discord-mock\"]"] },
        });
        session.sent.push((session.seq, payload.clone()));
        Gateway::queue(shared, &self.sender, payload, compress);
        drop(hub);
        self.session = Some(session_id);
        Ok(())
    }
}
