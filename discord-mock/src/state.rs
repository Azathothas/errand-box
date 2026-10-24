//! The one store, and the JSON projections both the REST answers and the
//! gateway dispatches are made from.
//!
//! Every mutation returns the dispatches it causes. The caller sends them
//! while it still holds the store, so the order of the events a client sees
//! is the order of the writes, and a REST answer and the event about it are
//! two projections of the same record (SimCord's rule, `SilentHacks/simcord@69bb92dcb458
//! docs/architecture.md:31-46`).
//!
//! Discord's rules are coded where they apply, each with its source in
//! `discord/discord-api-docs@6128d99116af`, abbreviated `docs` below.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::Config;
use crate::snowflake::{Snowflakes, iso, now_ms, unix_ms_of};

/// The intent bits a dispatch needs (docs `developers/events/gateway.mdx:313-392`).
pub mod intent {
    /// Always sent: READY, RESUMED, INTERACTION_CREATE.
    pub const NONE: u64 = 0;
    pub const GUILDS: u64 = 1;
    pub const GUILD_MESSAGES: u64 = 1 << 9;
    pub const GUILD_MESSAGE_REACTIONS: u64 = 1 << 10;
    pub const GUILD_MESSAGE_TYPING: u64 = 1 << 11;
    /// Privileged; it gates message content, not events (`gateway.mdx:487-503`).
    pub const MESSAGE_CONTENT: u64 = 1 << 15;
}

/// Message flags the mock reads (docs `developers/resources/message.mdx`,
/// "Message Flags").
pub mod flag {
    pub const SUPPRESS_EMBEDS: u64 = 1 << 2;
    pub const EPHEMERAL: u64 = 1 << 6;
    pub const LOADING: u64 = 1 << 7;
    pub const SUPPRESS_NOTIFICATIONS: u64 = 1 << 12;
}

/// Channel types (docs `developers/resources/channel.mdx`, "Channel Types").
pub const GUILD_TEXT: u8 = 0;
pub const PUBLIC_THREAD: u8 = 11;

/// The auto-archive durations Discord accepts, in minutes.
pub const AUTO_ARCHIVE_DURATIONS: [u64; 4] = [60, 1440, 4320, 10080];

/// An interaction token is valid for 15 minutes, and the first answer must
/// come within 3 seconds (docs `developers/interactions/receiving-and-responding.mdx:351`).
pub const INTERACTION_TOKEN_MS: u64 = 15 * 60 * 1000;
pub const INTERACTION_ANSWER_MS: u64 = 3000;

/// An event the store's change causes, before any session sees it.
#[derive(Clone, Debug)]
pub struct Dispatch {
    pub event: &'static str,
    pub data: Value,
    /// The intent a session needs to receive it; [`intent::NONE`] for all.
    pub intent: u64,
}

/// Discord's refusal of a request, as status, JSON error code and message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub status: u16,
    pub code: u64,
    pub message: &'static str,
}

/// The refusals the store gives, each in Discord's wording
/// (docs `developers/topics/opcodes-and-status-codes.mdx`, line given).
pub mod refusal {
    use super::Refusal;

    /// `:143`
    pub const UNKNOWN_CHANNEL: Refusal = Refusal {
        status: 404,
        code: 10003,
        message: "Unknown Channel",
    };
    /// `:148`
    pub const UNKNOWN_MESSAGE: Refusal = Refusal {
        status: 404,
        code: 10008,
        message: "Unknown Message",
    };
    /// `:155`
    pub const UNKNOWN_WEBHOOK: Refusal = Refusal {
        status: 404,
        code: 10015,
        message: "Unknown Webhook",
    };
    /// `:175`
    pub const UNKNOWN_INTERACTION: Refusal = Refusal {
        status: 404,
        code: 10062,
        message: "Unknown interaction",
    };
    /// `:252`
    pub const ALREADY_ACKNOWLEDGED: Refusal = Refusal {
        status: 400,
        code: 40060,
        message: "Interaction has already been acknowledged.",
    };
    /// `:266`
    pub const NOT_THE_AUTHOR: Refusal = Refusal {
        status: 403,
        code: 50005,
        message: "Cannot edit a message authored by another user",
    };
    /// `:289`
    pub const INVALID_FORM_BODY: Refusal = Refusal {
        status: 400,
        code: 50035,
        message: "Invalid Form Body",
    };
    /// `:305`
    pub const ARCHIVED: Refusal = Refusal {
        status: 400,
        code: 50083,
        message: "Thread is archived",
    };
    /// `:344`
    pub const THREAD_EXISTS: Refusal = Refusal {
        status: 400,
        code: 160_004,
        message: "A thread has already been created for this message",
    };
}

/// A person or the bot.
#[derive(Clone, Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub global_name: Option<String>,
    pub bot: bool,
}

/// A text channel or a thread.
#[derive(Clone, Debug)]
pub struct Channel {
    pub id: u64,
    pub kind: u8,
    pub name: String,
    pub parent_id: Option<u64>,
    pub owner_id: Option<u64>,
    pub last_message_id: Option<u64>,
    pub archived: bool,
    pub locked: bool,
    pub auto_archive_duration: u64,
    pub archive_ms: Option<u64>,
    pub created_ms: u64,
    /// Messages ever sent in a thread; Discord's `total_message_sent`.
    pub sent: u64,
    pub members: BTreeSet<u64>,
}

/// A message, by the facts its projections are made from.
#[derive(Clone, Debug)]
pub struct Message {
    pub id: u64,
    pub channel_id: u64,
    pub author_id: u64,
    pub kind: u8,
    pub content: String,
    pub flags: u64,
    pub edited_ms: Option<u64>,
    pub attachments: Vec<u64>,
    /// Each emoji with who reacted, in the order the emoji first came.
    pub reactions: Vec<(String, BTreeSet<u64>)>,
    pub reference: Option<u64>,
    pub thread_id: Option<u64>,
    /// Set on an interaction's answer: the interaction it answers.
    pub interaction: Option<u64>,
    pub deleted: bool,
}

/// A signed CDN URL's three query values (docs `developers/reference.mdx:415-424`).
#[derive(Clone, Debug)]
pub struct Signature {
    /// Expiry, issue, as Unix seconds.
    pub ex: u64,
    pub is: u64,
    /// Random, kept here: the mock checks it by comparison, not by HMAC.
    pub hm: String,
}

/// An uploaded file.
#[derive(Debug)]
pub struct Attachment {
    pub id: u64,
    pub channel_id: u64,
    pub filename: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
    /// Re-signed when a projection finds it expired, as Discord re-signs a
    /// URL in every payload it sends.
    pub signature: RefCell<Signature>,
}

/// Where an interaction's answer stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Pending,
    Deferred,
    Answered,
}

/// An application command a person used.
#[derive(Clone, Debug)]
pub struct Interaction {
    pub id: u64,
    pub token: String,
    pub channel_id: u64,
    pub user_id: u64,
    pub data: Value,
    pub created_ms: u64,
    pub answer: Answer,
    pub ephemeral: bool,
    pub original: Option<u64>,
}

/// A file a message carries, before it is stored.
#[derive(Clone, Debug)]
pub struct Upload {
    pub filename: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

/// What a new message says.
#[derive(Clone, Debug, Default)]
pub struct NewMessage {
    pub content: String,
    pub flags: u64,
    pub reference: Option<u64>,
    pub uploads: Vec<Upload>,
}

/// What an edit changes; a field left `None` stays.
#[derive(Clone, Debug, Default)]
pub struct MessageEdit {
    pub content: Option<String>,
    pub flags: Option<u64>,
}

/// The guild, its people, its channels and their messages.
#[derive(Debug)]
pub struct Store {
    pub config: Config,
    pub base_url: String,
    ids: Snowflakes,
    pub users: BTreeMap<u64, User>,
    pub channels: BTreeMap<u64, Channel>,
    pub messages: BTreeMap<u64, Message>,
    pub attachments: BTreeMap<u64, Attachment>,
    /// Commands per guild, as answered, by name.
    pub commands: BTreeMap<u64, BTreeMap<String, Value>>,
    pub interactions: BTreeMap<u64, Interaction>,
}

/// Lower-case hex of `n` random bytes.
pub fn random_hex(n: usize) -> String {
    let mut bytes = vec![0u8; n];
    getrandom::fill(&mut bytes).expect("the system's random source");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether two secrets are equal, in time that does not depend on where they
/// differ.
pub fn same_secret(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

fn id(value: u64) -> Value {
    Value::String(value.to_string())
}

impl Store {
    /// A store seeded from the configuration: the guild, its text channel,
    /// the bot and the people.
    pub fn new(config: Config, base_url: String) -> Self {
        let mut users = BTreeMap::new();
        users.insert(
            config.bot_id,
            User {
                id: config.bot_id,
                username: config.bot_name.clone(),
                global_name: None,
                bot: true,
            },
        );
        for person in &config.people {
            users.insert(
                person.id,
                User {
                    id: person.id,
                    username: person.username.clone(),
                    global_name: person.global_name.clone(),
                    bot: false,
                },
            );
        }
        let mut channels = BTreeMap::new();
        channels.insert(
            config.channel_id,
            Channel {
                id: config.channel_id,
                kind: GUILD_TEXT,
                name: config.channel_name.clone(),
                parent_id: None,
                owner_id: None,
                last_message_id: None,
                archived: false,
                locked: false,
                auto_archive_duration: 1440,
                archive_ms: None,
                created_ms: unix_ms_of(config.channel_id),
                sent: 0,
                members: BTreeSet::new(),
            },
        );
        Self {
            config,
            base_url,
            ids: Snowflakes::default(),
            users,
            channels,
            messages: BTreeMap::new(),
            attachments: BTreeMap::new(),
            commands: BTreeMap::new(),
            interactions: BTreeMap::new(),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.ids.next()
    }

    // -- projections ---------------------------------------------------

    /// A user object, with every field the spec's `UserResponse` requires.
    pub fn user_json(&self, user_id: u64) -> Value {
        let Some(user) = self.users.get(&user_id) else {
            return Value::Null;
        };
        let mut value = json!({
            "id": id(user.id),
            "username": user.username,
            "avatar": null,
            "discriminator": "0",
            "public_flags": 0,
            "flags": 0,
            "banner": null,
            "accent_color": null,
            "global_name": user.global_name,
            "avatar_decoration_data": null,
            "collectibles": null,
            "primary_guild": null,
        });
        if user.bot {
            value["bot"] = Value::Bool(true);
        }
        value
    }

    /// A guild member, as the gateway attaches one to a guild message.
    pub fn member_json(&self, user_id: u64, with_user: bool) -> Value {
        let mut value = json!({
            "roles": [],
            "nick": null,
            "avatar": null,
            "banner": null,
            "premium_since": null,
            "joined_at": iso(unix_ms_of(self.config.guild_id)),
            "deaf": false,
            "mute": false,
            "flags": 0,
            "pending": false,
            "communication_disabled_until": null,
        });
        if with_user {
            value["user"] = self.user_json(user_id);
        }
        value
    }

    /// A channel or a thread, as `GET /channels/{id}` answers it.
    pub fn channel_json(&self, channel: &Channel) -> Value {
        let mut value = json!({
            "id": id(channel.id),
            "type": channel.kind,
            "guild_id": id(self.config.guild_id),
            "name": channel.name,
            "last_message_id": channel.last_message_id.map(id),
            "flags": 0,
            "rate_limit_per_user": 0,
        });
        if channel.kind == PUBLIC_THREAD {
            let fields = json!({
                "parent_id": channel.parent_id.map(id),
                "owner_id": channel.owner_id.map(id),
                "message_count": channel.sent,
                "member_count": channel.members.len(),
                "total_message_sent": channel.sent,
                "thread_metadata": {
                    "archived": channel.archived,
                    "archive_timestamp": channel.archive_ms.map(iso),
                    "auto_archive_duration": channel.auto_archive_duration,
                    "locked": channel.locked,
                    "create_timestamp": iso(channel.created_ms),
                },
            });
            merge(&mut value, fields);
        } else {
            let fields = json!({
                "parent_id": null,
                "position": 0,
                "topic": null,
                "nsfw": false,
                "permission_overwrites": [],
                "last_pin_timestamp": null,
            });
            merge(&mut value, fields);
        }
        value
    }

    /// The thread member record of `user_id` in `thread_id`.
    pub fn thread_member_json(&self, thread_id: u64, user_id: u64) -> Value {
        let joined = self
            .channels
            .get(&thread_id)
            .map_or_else(now_ms, |thread| thread.created_ms);
        json!({
            "id": id(thread_id),
            "user_id": id(user_id),
            "join_timestamp": iso(joined),
            "flags": 0,
        })
    }

    /// An attachment, its URL signed afresh when the old one has expired.
    fn attachment_json(&self, attachment: &Attachment) -> Value {
        let now = now_ms() / 1000;
        if attachment.signature.borrow().ex <= now {
            *attachment.signature.borrow_mut() = self.sign(now);
        }
        let url = self.attachment_url(attachment);
        let mut value = json!({
            "id": id(attachment.id),
            "filename": attachment.filename,
            "size": attachment.bytes.len(),
            "url": url,
            "proxy_url": url,
            "flags": 0,
        });
        if let Some(content_type) = &attachment.content_type {
            value["content_type"] = Value::String(content_type.clone());
        }
        value
    }

    /// A fresh signature, issued at `now` (Unix seconds).
    pub fn sign(&self, now: u64) -> Signature {
        Signature {
            ex: now + self.config.attachment_ttl_secs,
            is: now,
            hm: random_hex(32),
        }
    }

    /// The signed URL an attachment is served at.
    pub fn attachment_url(&self, attachment: &Attachment) -> String {
        let signature = attachment.signature.borrow();
        format!(
            "{}/attachments/{}/{}/{}?ex={:x}&is={:x}&hm={}&",
            self.base_url,
            attachment.channel_id,
            attachment.id,
            attachment.filename,
            signature.ex,
            signature.is,
            signature.hm
        )
    }

    /// A message as REST answers it.
    pub fn message_json(&self, message: &Message) -> Value {
        let attachments: Vec<Value> = message
            .attachments
            .iter()
            .filter_map(|attachment| self.attachments.get(attachment))
            .map(|attachment| self.attachment_json(attachment))
            .collect();
        let mentions: Vec<Value> = mentioned(&message.content)
            .into_iter()
            .filter(|user| self.users.contains_key(user))
            .map(|user| self.user_json(user))
            .collect();
        let mut value = json!({
            "type": message.kind,
            "content": message.content,
            "mentions": mentions,
            "mention_roles": [],
            "attachments": attachments,
            "embeds": [],
            "timestamp": iso(unix_ms_of(message.id)),
            "edited_timestamp": message.edited_ms.map(iso),
            "flags": message.flags,
            "components": [],
            "id": id(message.id),
            "channel_id": id(message.channel_id),
            "author": self.user_json(message.author_id),
            "pinned": false,
            "mention_everyone": false,
            "tts": false,
        });
        if !message.reactions.is_empty() {
            value["reactions"] = Value::Array(
                message
                    .reactions
                    .iter()
                    .map(|(emoji, users)| {
                        json!({
                            "emoji": emoji_json(emoji),
                            "count": users.len(),
                            "count_details": { "burst": 0, "normal": users.len() },
                            "burst_colors": [],
                            "me_burst": false,
                            "burst_me": false,
                            "me": users.contains(&self.config.bot_id),
                            "burst_count": 0,
                        })
                    })
                    .collect(),
            );
        }
        if let Some(reference) = message.reference {
            value["message_reference"] = json!({
                "type": 0,
                "channel_id": id(message.channel_id),
                "message_id": id(reference),
                "guild_id": id(self.config.guild_id),
            });
            value["referenced_message"] = self
                .messages
                .get(&reference)
                .filter(|referenced| !referenced.deleted)
                .map_or(Value::Null, |referenced| self.message_json(referenced));
        }
        if let Some(thread) = message
            .thread_id
            .and_then(|thread| self.channels.get(&thread))
        {
            value["thread"] = self.channel_json(thread);
        }
        if let Some(interaction) = message
            .interaction
            .and_then(|interaction| self.interactions.get(&interaction))
        {
            value["webhook_id"] = id(self.config.application_id);
            value["application_id"] = id(self.config.application_id);
            value["interaction_metadata"] = json!({
                "id": id(interaction.id),
                "type": 2,
                "user": self.user_json(interaction.user_id),
                "authorizing_integration_owners": { "0": id(self.config.guild_id) },
                "name": interaction.data["name"],
                "command_type": 1,
            });
        }
        if self
            .channels
            .get(&message.channel_id)
            .is_some_and(|channel| channel.kind == PUBLIC_THREAD)
        {
            let position = self
                .messages
                .values()
                .filter(|other| other.channel_id == message.channel_id && other.id < message.id)
                .count();
            value["position"] = json!(position);
        }
        value
    }

    /// A message as the gateway sends it: the REST shape, plus the guild and
    /// the author's membership.
    pub fn message_event_json(&self, message: &Message) -> Value {
        let mut value = self.message_json(message);
        value["guild_id"] = id(self.config.guild_id);
        value["member"] = self.member_json(message.author_id, false);
        value
    }

    /// The guild, as GUILD_CREATE carries it after READY.
    pub fn guild_json(&self) -> Value {
        let channels: Vec<Value> = self
            .channels
            .values()
            .filter(|channel| channel.kind == GUILD_TEXT)
            .map(|channel| self.channel_json(channel))
            .collect();
        let threads: Vec<Value> = self
            .channels
            .values()
            .filter(|channel| channel.kind == PUBLIC_THREAD && !channel.archived)
            .map(|channel| self.channel_json(channel))
            .collect();
        let guild = id(self.config.guild_id);
        let mut members = vec![self.member_json(self.config.bot_id, true)];
        members.extend(
            self.config
                .people
                .iter()
                .map(|person| self.member_json(person.id, true)),
        );
        json!({
            "id": guild,
            "name": self.config.guild_name,
            "icon": null,
            "icon_hash": null,
            "splash": null,
            "discovery_splash": null,
            "owner_id": id(self.config.people.first().map_or(self.config.bot_id, |person| person.id)),
            "afk_channel_id": null,
            "afk_timeout": 300,
            "widget_enabled": false,
            "widget_channel_id": null,
            "verification_level": 0,
            "default_message_notifications": 0,
            "explicit_content_filter": 0,
            "roles": [{
                "id": guild,
                "name": "@everyone",
                "description": null,
                "permissions": "2248473465835073",
                "position": 0,
                "color": 0,
                "colors": { "primary_color": 0, "secondary_color": null, "tertiary_color": null },
                "hoist": false,
                "managed": false,
                "mentionable": false,
                "icon": null,
                "unicode_emoji": null,
                "flags": 0,
            }],
            "emojis": [],
            "features": [],
            "mfa_level": 0,
            "application_id": null,
            "system_channel_id": null,
            "system_channel_flags": 0,
            "rules_channel_id": null,
            "max_presences": null,
            "max_members": 25_000_000,
            "vanity_url_code": null,
            "description": null,
            "banner": null,
            "premium_tier": 0,
            "premium_subscription_count": 0,
            "preferred_locale": "en-US",
            "public_updates_channel_id": null,
            "max_video_channel_users": 25,
            "max_stage_video_channel_users": 50,
            "nsfw_level": 0,
            "stickers": [],
            "premium_progress_bar_enabled": false,
            "safety_alerts_channel_id": null,
            "incidents_data": null,
            "joined_at": iso(unix_ms_of(self.config.guild_id)),
            "large": false,
            "unavailable": false,
            "member_count": members.len(),
            "voice_states": [],
            "members": members,
            "channels": channels,
            "threads": threads,
            "presences": [],
            "stage_instances": [],
            "guild_scheduled_events": [],
            "soundboard_sounds": [],
            "lazy": true,
        })
    }

    /// The application command a person can use, as the interaction carries
    /// it; none when no command of that name is registered.
    pub fn command(&self, name: &str) -> Option<&Value> {
        self.commands.get(&self.config.guild_id)?.get(name)
    }

    // -- lookups -------------------------------------------------------

    pub fn channel(&self, channel_id: u64) -> Result<&Channel, Refusal> {
        self.channels
            .get(&channel_id)
            .ok_or(refusal::UNKNOWN_CHANNEL)
    }

    fn message_in(&self, channel_id: u64, message_id: u64) -> Result<&Message, Refusal> {
        self.channel(channel_id)?;
        self.messages
            .get(&message_id)
            .filter(|message| message.channel_id == channel_id && !message.deleted)
            .ok_or(refusal::UNKNOWN_MESSAGE)
    }

    pub fn get_message(&self, channel_id: u64, message_id: u64) -> Result<Value, Refusal> {
        Ok(self.message_json(self.message_in(channel_id, message_id)?))
    }

    /// Refuses an operation an archived thread does not allow: editing a
    /// message, adding a reaction (docs `developers/topics/threads.mdx:59`).
    fn refuse_if_archived(&self, channel_id: u64) -> Result<(), Refusal> {
        if self.channel(channel_id)?.archived {
            return Err(refusal::ARCHIVED);
        }
        Ok(())
    }

    // -- mutations -----------------------------------------------------

    /// THREAD_UPDATE, then THREAD_MEMBER_UPDATE: what unarchiving a thread
    /// sends, in this order (docs `developers/topics/threads.mdx:198-203`).
    fn unarchive(&mut self, thread_id: u64, events: &mut Vec<Dispatch>) {
        let Some(thread) = self.channels.get_mut(&thread_id) else {
            return;
        };
        thread.archived = false;
        thread.archive_ms = Some(now_ms());
        let thread = thread.clone();
        events.push(Dispatch {
            event: "THREAD_UPDATE",
            data: self.channel_json(&thread),
            intent: intent::GUILDS,
        });
        if thread.members.contains(&self.config.bot_id) {
            events.push(self.thread_member_update(thread_id));
        }
    }

    fn thread_member_update(&self, thread_id: u64) -> Dispatch {
        let mut data = self.thread_member_json(thread_id, self.config.bot_id);
        data["guild_id"] = id(self.config.guild_id);
        Dispatch {
            event: "THREAD_MEMBER_UPDATE",
            data,
            intent: intent::GUILDS,
        }
    }

    /// Posts a message as `author_id`, the bot or a person.
    ///
    /// A message in an archived thread unarchives it, unless it is locked
    /// (docs `developers/topics/threads.mdx:59`). The poster joins a thread,
    /// as Discord adds a person who writes there.
    pub fn create_message(
        &mut self,
        channel_id: u64,
        author_id: u64,
        new: NewMessage,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        let channel = self.channel(channel_id)?.clone();
        if let Some(reference) = new.reference
            && self.message_in(channel_id, reference).is_err()
        {
            return Err(refusal::INVALID_FORM_BODY);
        }
        let mut events = Vec::new();
        if channel.archived {
            if channel.locked {
                return Err(refusal::ARCHIVED);
            }
            self.unarchive(channel_id, &mut events);
        }
        let message_id = self.next_id();
        let mut attachments = Vec::new();
        for upload in new.uploads {
            let attachment_id = self.next_id();
            let signature = self.sign(now_ms() / 1000);
            self.attachments.insert(
                attachment_id,
                Attachment {
                    id: attachment_id,
                    channel_id,
                    filename: upload.filename,
                    content_type: upload.content_type,
                    bytes: upload.bytes,
                    signature: RefCell::new(signature),
                },
            );
            attachments.push(attachment_id);
        }
        let message = Message {
            id: message_id,
            channel_id,
            author_id,
            kind: if new.reference.is_some() { 19 } else { 0 },
            content: new.content,
            flags: new.flags,
            edited_ms: None,
            attachments,
            reactions: Vec::new(),
            reference: new.reference,
            thread_id: None,
            interaction: None,
            deleted: false,
        };
        self.messages.insert(message_id, message.clone());
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.last_message_id = Some(message_id);
            if channel.kind == PUBLIC_THREAD {
                channel.sent += 1;
                channel.members.insert(author_id);
            }
        }
        events.push(Dispatch {
            event: "MESSAGE_CREATE",
            data: self.message_event_json(&message),
            intent: intent::GUILD_MESSAGES,
        });
        Ok((self.message_json(&message), events))
    }

    /// Edits the bot's own message.
    pub fn edit_message(
        &mut self,
        channel_id: u64,
        message_id: u64,
        edit: MessageEdit,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        let message = self.message_in(channel_id, message_id)?;
        if message.author_id != self.config.bot_id {
            return Err(refusal::NOT_THE_AUTHOR);
        }
        self.refuse_if_archived(channel_id)?;
        let message = self
            .messages
            .get_mut(&message_id)
            .expect("found just above");
        if let Some(content) = edit.content {
            message.content = content;
        }
        if let Some(flags) = edit.flags {
            message.flags = flags;
        }
        message.edited_ms = Some(now_ms());
        let message = message.clone();
        let events = vec![Dispatch {
            event: "MESSAGE_UPDATE",
            data: self.message_event_json(&message),
            intent: intent::GUILD_MESSAGES,
        }];
        Ok((self.message_json(&message), events))
    }

    /// Deletes a message; allowed in an archived thread
    /// (docs `developers/topics/threads.mdx:59`).
    pub fn delete_message(
        &mut self,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Vec<Dispatch>, Refusal> {
        self.message_in(channel_id, message_id)?;
        if let Some(message) = self.messages.get_mut(&message_id) {
            message.deleted = true;
        }
        Ok(vec![Dispatch {
            event: "MESSAGE_DELETE",
            data: json!({
                "id": id(message_id),
                "channel_id": id(channel_id),
                "guild_id": id(self.config.guild_id),
            }),
            intent: intent::GUILD_MESSAGES,
        }])
    }

    /// Adds or removes `user_id`'s reaction. Adding one that is there, or
    /// removing one that is not, changes nothing and sends nothing, as
    /// Discord answers both with 204.
    pub fn react(
        &mut self,
        channel_id: u64,
        message_id: u64,
        emoji: &str,
        user_id: u64,
        add: bool,
    ) -> Result<Vec<Dispatch>, Refusal> {
        self.message_in(channel_id, message_id)?;
        self.refuse_if_archived(channel_id)?;
        let message = self
            .messages
            .get_mut(&message_id)
            .expect("found just above");
        let changed = if add {
            match message.reactions.iter_mut().find(|(name, _)| name == emoji) {
                Some((_, users)) => users.insert(user_id),
                None => {
                    message
                        .reactions
                        .push((emoji.to_owned(), BTreeSet::from([user_id])));
                    true
                }
            }
        } else {
            let removed = message
                .reactions
                .iter_mut()
                .find(|(name, _)| name == emoji)
                .is_some_and(|(_, users)| users.remove(&user_id));
            message.reactions.retain(|(_, users)| !users.is_empty());
            removed
        };
        let author_id = message.author_id;
        if !changed {
            return Ok(Vec::new());
        }
        let mut data = json!({
            "user_id": id(user_id),
            "type": 0,
            "message_id": id(message_id),
            "channel_id": id(channel_id),
            "guild_id": id(self.config.guild_id),
            "emoji": emoji_json(emoji),
            "burst": false,
        });
        if add {
            data["member"] = self.member_json(user_id, true);
            data["message_author_id"] = id(author_id);
            data["burst_colors"] = json!([]);
        }
        Ok(vec![Dispatch {
            event: if add {
                "MESSAGE_REACTION_ADD"
            } else {
                "MESSAGE_REACTION_REMOVE"
            },
            data,
            intent: intent::GUILD_MESSAGE_REACTIONS,
        }])
    }

    /// Shows `user_id` typing in a channel.
    pub fn typing(&self, channel_id: u64, user_id: u64) -> Result<Vec<Dispatch>, Refusal> {
        self.channel(channel_id)?;
        Ok(vec![Dispatch {
            event: "TYPING_START",
            data: json!({
                "channel_id": id(channel_id),
                "guild_id": id(self.config.guild_id),
                "user_id": id(user_id),
                "timestamp": now_ms() / 1000,
                "member": self.member_json(user_id, true),
            }),
            intent: intent::GUILD_MESSAGE_TYPING,
        }])
    }

    /// Starts a public thread from a message in a text channel.
    ///
    /// The thread's id is the message's id, so a message has one thread at
    /// most (docs `developers/resources/channel.mdx:612-614`). It sends
    /// THREAD_CREATE and MESSAGE_UPDATE (`:612`), then THREAD_MEMBER_UPDATE
    /// for the bot, which joins the thread it made: inferred from
    /// `developers/events/gateway-events.mdx:637-642`, not captured.
    pub fn create_thread(
        &mut self,
        channel_id: u64,
        message_id: u64,
        name: String,
        auto_archive_duration: u64,
        creator: u64,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        if self.channel(channel_id)?.kind != GUILD_TEXT {
            return Err(refusal::INVALID_FORM_BODY);
        }
        let starter = self.message_in(channel_id, message_id)?;
        if starter.thread_id.is_some() || self.channels.contains_key(&message_id) {
            return Err(refusal::THREAD_EXISTS);
        }
        let thread = Channel {
            id: message_id,
            kind: PUBLIC_THREAD,
            name,
            parent_id: Some(channel_id),
            owner_id: Some(creator),
            last_message_id: None,
            archived: false,
            locked: false,
            auto_archive_duration,
            archive_ms: Some(now_ms()),
            created_ms: now_ms(),
            sent: 0,
            members: BTreeSet::from([creator]),
        };
        self.channels.insert(message_id, thread.clone());
        let starter = self
            .messages
            .get_mut(&message_id)
            .expect("found just above");
        starter.thread_id = Some(message_id);
        let starter = starter.clone();
        let mut created = self.channel_json(&thread);
        created["newly_created"] = Value::Bool(true);
        let mut events = vec![
            Dispatch {
                event: "THREAD_CREATE",
                data: created,
                intent: intent::GUILDS,
            },
            Dispatch {
                event: "MESSAGE_UPDATE",
                data: self.message_event_json(&starter),
                intent: intent::GUILD_MESSAGES,
            },
        ];
        if creator == self.config.bot_id {
            events.push(self.thread_member_update(message_id));
        }
        Ok((self.channel_json(&thread), events))
    }

    /// Archives, unarchives, locks or unlocks a thread.
    pub fn edit_thread(
        &mut self,
        thread_id: u64,
        archived: Option<bool>,
        locked: Option<bool>,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        let thread = self.channel(thread_id)?;
        if thread.kind != PUBLIC_THREAD {
            return Err(refusal::INVALID_FORM_BODY);
        }
        let was_archived = thread.archived;
        let mut events = Vec::new();
        if archived == Some(false) && was_archived {
            self.unarchive(thread_id, &mut events);
        }
        let thread = self.channels.get_mut(&thread_id).expect("found just above");
        if let Some(locked) = locked {
            thread.locked = locked;
        }
        if archived == Some(true) && !was_archived {
            thread.archived = true;
            thread.archive_ms = Some(now_ms());
        }
        let thread = thread.clone();
        let changed = locked.is_some() || (archived == Some(true) && !was_archived);
        if changed {
            // An unarchive already sent its THREAD_UPDATE; this one carries
            // the rest of the change.
            events.push(Dispatch {
                event: "THREAD_UPDATE",
                data: self.channel_json(&thread),
                intent: intent::GUILDS,
            });
        }
        Ok((self.channel_json(&thread), events))
    }

    /// Deletes a thread, as a moderator does from outside.
    pub fn delete_thread(&mut self, thread_id: u64) -> Result<Vec<Dispatch>, Refusal> {
        let thread = self.channel(thread_id)?.clone();
        if thread.kind != PUBLIC_THREAD {
            return Err(refusal::INVALID_FORM_BODY);
        }
        self.channels.remove(&thread_id);
        Ok(vec![Dispatch {
            event: "THREAD_DELETE",
            data: json!({
                "id": id(thread_id),
                "guild_id": id(self.config.guild_id),
                "parent_id": thread.parent_id.map(id),
                "type": PUBLIC_THREAD,
            }),
            intent: intent::GUILDS,
        }])
    }

    /// Replaces the guild's commands, keeping the id of a command whose name
    /// stays.
    pub fn put_commands(&mut self, guild_id: u64, wanted: Vec<Map<String, Value>>) -> Value {
        let old = self.commands.remove(&guild_id).unwrap_or_default();
        let mut now = BTreeMap::new();
        let mut answered = Vec::new();
        for mut command in wanted {
            let name = command
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let command_id = old
                .get(&name)
                .and_then(|command| command["id"].as_str())
                .and_then(|id| id.parse().ok())
                .unwrap_or_else(|| self.next_id());
            command.insert("id".into(), id(command_id));
            command.insert("application_id".into(), id(self.config.application_id));
            command.insert("guild_id".into(), id(guild_id));
            command.insert("version".into(), id(self.next_id()));
            command.entry("type").or_insert(json!(1));
            command.entry("description").or_insert(json!(""));
            command.entry("options").or_insert(json!([]));
            command.entry("nsfw").or_insert(json!(false));
            command.entry("integration_types").or_insert(json!([0]));
            command.entry("contexts").or_insert(Value::Null);
            // Discord answers the permission bits as a string or null.
            let permissions = match command.get("default_member_permissions") {
                Some(Value::Number(bits)) => Value::String(bits.to_string()),
                Some(Value::String(bits)) => Value::String(bits.clone()),
                _ => Value::Null,
            };
            command.insert("default_member_permissions".into(), permissions);
            command.remove("dm_permission");
            let command = Value::Object(command);
            answered.push(command.clone());
            now.insert(name, command);
        }
        self.commands.insert(guild_id, now);
        Value::Array(answered)
    }

    /// A person uses an application command, by name and options.
    pub fn use_command(
        &mut self,
        name: &str,
        options: Value,
        channel_id: u64,
        user_id: u64,
    ) -> Result<(u64, String, Vec<Dispatch>), Refusal> {
        let channel = self.channel(channel_id)?.clone();
        // `opcodes-and-status-codes.mdx:176`
        let command = self.command(name).cloned().ok_or(Refusal {
            status: 404,
            code: 10063,
            message: "Unknown application command",
        })?;
        let interaction_id = self.next_id();
        let token = format!("mock.interaction.{}", random_hex(24));
        let data = json!({
            "id": command["id"],
            "name": name,
            "type": 1,
            "options": options,
            "guild_id": id(self.config.guild_id),
        });
        let mut member = self.member_json(user_id, true);
        member["permissions"] = Value::String("2248473465835073".into());
        let mut channel_value = self.channel_json(&channel);
        channel_value["permissions"] = Value::String("2248473465835073".into());
        let event = json!({
            "id": id(interaction_id),
            "application_id": id(self.config.application_id),
            "type": 2,
            "data": data,
            "guild_id": id(self.config.guild_id),
            "guild": { "id": id(self.config.guild_id), "locale": "en-US", "features": [] },
            "channel_id": id(channel_id),
            "channel": channel_value,
            "member": member,
            "token": token,
            "version": 1,
            "app_permissions": "2248473465835073",
            "locale": "en-US",
            "guild_locale": "en-US",
            "entitlements": [],
            "authorizing_integration_owners": { "0": id(self.config.guild_id) },
            "context": 0,
            "attachment_size_limit": 10_485_760,
        });
        self.interactions.insert(
            interaction_id,
            Interaction {
                id: interaction_id,
                token: token.clone(),
                channel_id,
                user_id,
                data,
                created_ms: now_ms(),
                answer: Answer::Pending,
                ephemeral: false,
                original: None,
            },
        );
        Ok((
            interaction_id,
            token,
            vec![Dispatch {
                event: "INTERACTION_CREATE",
                data: event,
                intent: intent::NONE,
            }],
        ))
    }

    /// The interaction a URL's id and token name, while its token is valid.
    fn interaction_by_token(&self, token: &str) -> Result<&Interaction, Refusal> {
        let found = self
            .interactions
            .values()
            .find(|interaction| same_secret(&interaction.token, token))
            .ok_or(refusal::UNKNOWN_WEBHOOK)?;
        if now_ms().saturating_sub(found.created_ms) > INTERACTION_TOKEN_MS {
            return Err(refusal::UNKNOWN_WEBHOOK);
        }
        Ok(found)
    }

    /// Posts an interaction's message: the answer or a follow-up.
    fn post_for(
        &mut self,
        interaction_id: u64,
        content: String,
        flags: u64,
    ) -> (u64, Vec<Dispatch>) {
        let interaction = self.interactions[&interaction_id].clone();
        let message_id = self.next_id();
        let message = Message {
            id: message_id,
            channel_id: interaction.channel_id,
            author_id: self.config.bot_id,
            kind: 20,
            content,
            flags,
            edited_ms: None,
            attachments: Vec::new(),
            reactions: Vec::new(),
            reference: None,
            thread_id: None,
            interaction: Some(interaction_id),
            deleted: false,
        };
        self.messages.insert(message_id, message.clone());
        // An ephemeral message is seen only by the person who asked; the
        // gateway does not carry it to the bot.
        let events = if flags & flag::EPHEMERAL == 0 {
            vec![Dispatch {
                event: "MESSAGE_CREATE",
                data: self.message_event_json(&message),
                intent: intent::GUILD_MESSAGES,
            }]
        } else {
            Vec::new()
        };
        (message_id, events)
    }

    /// The first answer to an interaction: type 4 with a message, or type 5,
    /// deferred. It must come within 3 seconds, and only once.
    pub fn answer_interaction(
        &mut self,
        interaction_id: u64,
        token: &str,
        kind: u64,
        content: Option<String>,
        flags: u64,
    ) -> Result<Vec<Dispatch>, Refusal> {
        let interaction = self
            .interactions
            .get(&interaction_id)
            .filter(|interaction| same_secret(&interaction.token, token))
            .ok_or(refusal::UNKNOWN_INTERACTION)?;
        if interaction.answer != Answer::Pending {
            return Err(refusal::ALREADY_ACKNOWLEDGED);
        }
        if now_ms().saturating_sub(interaction.created_ms) > INTERACTION_ANSWER_MS {
            return Err(refusal::UNKNOWN_INTERACTION);
        }
        let ephemeral = flags & flag::EPHEMERAL != 0;
        let mut events = Vec::new();
        let (answer, original) = match kind {
            4 => {
                let (message, posted) =
                    self.post_for(interaction_id, content.unwrap_or_default(), flags);
                events = posted;
                (Answer::Answered, Some(message))
            }
            5 if ephemeral => (Answer::Deferred, None),
            5 => {
                let (message, posted) =
                    self.post_for(interaction_id, String::new(), flags | flag::LOADING);
                events = posted;
                (Answer::Deferred, Some(message))
            }
            _ => return Err(refusal::INVALID_FORM_BODY),
        };
        let interaction = self
            .interactions
            .get_mut(&interaction_id)
            .expect("found just above");
        interaction.answer = answer;
        interaction.ephemeral = ephemeral;
        interaction.original = original;
        Ok(events)
    }

    /// Edits the original answer; for a deferred ephemeral answer, this is
    /// what posts it.
    pub fn edit_original(
        &mut self,
        token: &str,
        content: Option<String>,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        let interaction = self.interaction_by_token(token)?.clone();
        if interaction.answer == Answer::Pending {
            return Err(refusal::UNKNOWN_MESSAGE);
        }
        let flags = if interaction.ephemeral {
            flag::EPHEMERAL
        } else {
            0
        };
        let Some(original) = interaction.original else {
            let (message_id, events) =
                self.post_for(interaction.id, content.unwrap_or_default(), flags);
            if let Some(stored) = self.interactions.get_mut(&interaction.id) {
                stored.original = Some(message_id);
                stored.answer = Answer::Answered;
            }
            return Ok((self.message_json(&self.messages[&message_id]), events));
        };
        let message = self
            .messages
            .get_mut(&original)
            .expect("an original answer");
        if let Some(content) = content {
            message.content = content;
        }
        message.flags &= !flag::LOADING;
        message.edited_ms = Some(now_ms());
        let message = message.clone();
        let events = if interaction.ephemeral {
            Vec::new()
        } else {
            vec![Dispatch {
                event: "MESSAGE_UPDATE",
                data: self.message_event_json(&message),
                intent: intent::GUILD_MESSAGES,
            }]
        };
        if let Some(stored) = self.interactions.get_mut(&interaction.id) {
            stored.answer = Answer::Answered;
        }
        Ok((self.message_json(&message), events))
    }

    /// A follow-up message to an interaction already answered or deferred.
    pub fn follow_up(
        &mut self,
        token: &str,
        content: String,
        flags: u64,
    ) -> Result<(Value, Vec<Dispatch>), Refusal> {
        let interaction = self.interaction_by_token(token)?.clone();
        if interaction.answer == Answer::Pending {
            return Err(refusal::UNKNOWN_WEBHOOK);
        }
        let (message_id, events) = self.post_for(interaction.id, content, flags);
        Ok((self.message_json(&self.messages[&message_id]), events))
    }
}

/// Copies every key of `extra` into `value`.
fn merge(value: &mut Value, extra: Value) {
    if let (Value::Object(into), Value::Object(from)) = (value, extra) {
        into.extend(from);
    }
}

/// The emoji object of a reaction: a Unicode emoji by name, or a custom one
/// written `name:id`, as a path segment names it.
pub fn emoji_json(emoji: &str) -> Value {
    match emoji.split_once(':') {
        Some((name, custom)) if custom.parse::<u64>().is_ok() => {
            json!({ "id": custom, "name": name })
        }
        _ => json!({ "id": null, "name": emoji }),
    }
}

/// The users a message mentions, by the `<@id>` and `<@!id>` forms.
pub fn mentioned(content: &str) -> Vec<u64> {
    let mut found = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("<@") {
        rest = &rest[start + 2..];
        let digits = rest.strip_prefix('!').unwrap_or(rest);
        let end = digits
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(digits.len());
        if digits[end..].starts_with('>')
            && let Ok(user) = digits[..end].parse::<u64>()
            && !found.contains(&user)
        {
            found.push(user);
        }
    }
    found
}
