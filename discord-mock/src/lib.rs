//! A local stand-in for the part of Discord that errand reaches, for tests.
//!
//! One store with two projections: every REST write and every event a test
//! injects is a change to the store, and the change returns the gateway
//! dispatches it causes, so an answer and the event about it cannot
//! disagree. A route, a method or a body field the mock does not implement
//! answers 501 and is journalled as a gap, and a [`MockHandle`] dropped with
//! an unread gap fails the test that dropped it.
//!
//! What is coded follows Discord's documentation at
//! `discord/discord-api-docs@6128d99116af`, cited where each rule is, and the
//! REST answers are checked against `discord/discord-api-spec` by
//! `scripts/discord-spec.sh --check`. The mock serves plain `http://` and
//! `ws://` on loopback; errand reaches it only through the guarded switch in
//! `errand/src/chat/endpoint.rs`.

// The guild object of GUILD_CREATE is one `json!` with more keys than the
// default limit expands.
#![recursion_limit = "256"]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

mod control;
mod gateway;
mod journal;
mod rest;
mod snowflake;
mod state;

pub use control::Injected;
pub use gateway::Fault;
pub use journal::{Entry, Journal};
pub use state::Refusal;

/// A person in the guild.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Person {
    pub id: u64,
    pub username: String,
    #[serde(default)]
    pub global_name: Option<String>,
}

/// The guild the mock serves, and how its gateway behaves.
///
/// Read from a seed file by the binary, so every key has a default and an
/// unknown key is refused.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// The bot token, without `Bot `. errand's guard takes only a token that
    /// begins with `mock.`.
    pub token: String,
    pub application_id: u64,
    pub bot_id: u64,
    pub bot_name: String,
    pub guild_id: u64,
    pub guild_name: String,
    /// The one text channel, the channel errand serves.
    pub channel_id: u64,
    pub channel_name: String,
    pub people: Vec<Person>,
    /// HELLO's `heartbeat_interval`; Discord's is 41250 ms.
    pub heartbeat_interval_ms: u64,
    /// Whether the application may ask for the privileged content intent.
    /// Without it, an IDENTIFY that asks is closed with 4014.
    pub message_content_intent: bool,
    /// Whether a session that asked for payload compression gets its
    /// dispatches zlib-compressed.
    pub compress: bool,
    /// Client frames allowed per 60 seconds before close 4008; Discord's
    /// limit is 120 (`developers/events/gateway.mdx:509`).
    pub gateway_sends_per_minute: usize,
    /// How long a signed attachment URL lives.
    pub attachment_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        // Fixed ids, shaped as snowflakes from 2024, so a test can name them.
        // The bot's user id is its application id, as for an application
        // made since 2017.
        Self {
            token: "mock.bot-token".into(),
            application_id: 1_190_000_000_000_000_001,
            bot_id: 1_190_000_000_000_000_001,
            bot_name: "errand-mock".into(),
            guild_id: 1_190_000_000_000_000_010,
            guild_name: "discord-mock".into(),
            channel_id: 1_190_000_000_000_000_020,
            channel_name: "errand-test".into(),
            people: vec![Person {
                id: 1_190_000_000_000_000_100,
                username: "amelia".into(),
                global_name: Some("Amelia".into()),
            }],
            heartbeat_interval_ms: 41_250,
            message_content_intent: true,
            compress: true,
            gateway_sends_per_minute: 120,
            attachment_ttl_secs: 24 * 60 * 60,
        }
    }
}

/// What every task of one mock shares.
#[derive(Debug)]
pub(crate) struct Shared {
    pub config: Config,
    pub store: Mutex<state::Store>,
    pub journal: Journal,
    pub gateway: gateway::Gateway,
    pub ws_url: String,
}

/// The mock's entry point.
pub struct Mock;

impl Mock {
    /// Starts the REST listener on `listen` and the gateway on another port
    /// of the same address. `127.0.0.1:0` picks free ports.
    ///
    /// # Errors
    ///
    /// When either port cannot be bound.
    pub async fn start_on(config: Config, listen: SocketAddr) -> std::io::Result<MockHandle> {
        let rest = TcpListener::bind(listen).await?;
        let gateway = TcpListener::bind(SocketAddr::new(listen.ip(), 0)).await?;
        let base_url = format!("http://{}", rest.local_addr()?);
        let ws_url = format!("ws://{}", gateway.local_addr()?);
        let shared = Arc::new(Shared {
            store: Mutex::new(state::Store::new(config.clone(), base_url.clone())),
            config,
            journal: Journal::default(),
            gateway: gateway::Gateway::default(),
            ws_url: ws_url.clone(),
        });
        let app = rest::router(Arc::clone(&shared));
        let rest_task = tokio::spawn(async move {
            // Serving ends only when the task is aborted.
            let _ = axum::serve(rest, app).await;
        });
        let gateway_task = tokio::spawn(gateway::serve(Arc::clone(&shared), gateway));
        Ok(MockHandle {
            base_url,
            ws_url,
            shared,
            tasks: vec![rest_task, gateway_task],
            gaps_read: false,
        })
    }

    /// Starts on loopback, on free ports.
    ///
    /// # Errors
    ///
    /// When a port cannot be bound.
    pub async fn start(config: Config) -> std::io::Result<MockHandle> {
        Self::start_on(config, SocketAddr::from(([127, 0, 0, 1], 0))).await
    }
}

/// A running mock. Dropping it stops it, and fails the test when a gap was
/// recorded and not read with [`MockHandle::take_gaps`].
pub struct MockHandle {
    /// `http://127.0.0.1:PORT`: what `ERRAND_DISCORD_MOCK_URL` names.
    pub base_url: String,
    /// `ws://127.0.0.1:PORT`: what `GET /api/v10/gateway` answers.
    pub ws_url: String,
    shared: Arc<Shared>,
    tasks: Vec<JoinHandle<()>>,
    gaps_read: bool,
}

impl MockHandle {
    /// The configuration it runs with.
    pub fn config(&self) -> &Config {
        &self.shared.config
    }

    /// Everything the journal holds so far.
    pub fn journal(&self) -> Vec<Entry> {
        self.shared.journal.entries()
    }

    /// Waits until `found` answers for the journal, no longer than `limit`.
    ///
    /// # Errors
    ///
    /// When nothing matched in time; the message names the wait.
    pub async fn wait_for<T>(
        &self,
        limit: Duration,
        found: impl FnMut(&[Entry]) -> Option<T>,
    ) -> Result<T, String> {
        self.shared.journal.wait_for(limit, found).await
    }

    /// The gaps recorded so far, and the promise that they were read.
    pub fn take_gaps(&mut self) -> Vec<String> {
        self.gaps_read = true;
        self.shared.journal.gaps()
    }

    /// Applies a gateway fault to every open socket.
    pub fn fault(&self, fault: Fault) {
        self.shared.gateway.fault(&self.shared, fault);
    }
}

impl Drop for MockHandle {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
        let gaps = self.shared.journal.gaps();
        if !self.gaps_read && !gaps.is_empty() && !std::thread::panicking() {
            panic!(
                "discord-mock: errand reached what the mock does not implement:\n  {}",
                gaps.join("\n  ")
            );
        }
    }
}
