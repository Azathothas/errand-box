//! Where the chat client connects: Discord, or a local stand-in for tests.
//!
//! The stand-in is chosen by the environment variable
//! `ERRAND_DISCORD_MOCK_URL`, never by the configuration, so no file
//! errand-box writes can choose it. Unset, the client is built exactly as
//! before, with `serenity::Client::builder`.
//!
//! Set, it is honoured only when three conditions hold, and errand refuses
//! to start otherwise, before any request:
//!
//! 1. the URL is `http://` and its host is an IP literal on loopback
//!    (127.0.0.0/8 or `::1`), so no resolver decides where the token goes;
//! 2. the bot token begins with `mock.`, which no Discord bot token does: a
//!    Discord token begins with the bot's id in base64;
//! 3. the gateway URL the stand-in answers `GET /gateway` with is `ws://` on
//!    a loopback IP literal. serenity falls back to `wss://gateway.discord.gg`
//!    when that call fails (serenity 0.12.5 `src/client/mod.rs:364-370`),
//!    and this closes that fallback.
//!
//! On the stand-in, serenity's rate limiter is off: its limiter ignores the
//! base override (serenity 0.12.5 `src/http/ratelimiting.rs:199`), and the
//! stand-in never answers 429. The REST client is built with `.no_proxy()`,
//! because a proxy from the environment would carry the loopback request
//! away (hyper-util 0.1.20 `src/client/proxy/matcher.rs:122-125` has no
//! loopback exception).

use std::net::IpAddr;

use serenity::client::ClientBuilder;
use serenity::http::HttpBuilder;
use serenity::model::gateway::GatewayIntents;
use url::{Host, Url};

use crate::config::load::Environment;

/// The variable that names the stand-in's REST base URL.
pub const MOCK_VARIABLE: &str = "ERRAND_DISCORD_MOCK_URL";

/// What a stand-in's bot token begins with.
pub const MOCK_TOKEN_PREFIX: &str = "mock.";

/// Where the chat client connects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    /// Discord itself.
    Discord,
    /// A stand-in at this base URL, `http://127.0.0.1:PORT`.
    Mock { base: String },
}

/// Why the stand-in was refused. Each names the condition that failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("{MOCK_VARIABLE} is not a URL: {0}")]
    NotAUrl(String),
    #[error("{MOCK_VARIABLE} must be http://, not {0}://")]
    NotHttp(String),
    #[error(
        "{MOCK_VARIABLE} must name a loopback IP address, not {0}: a host name or another address could send the token elsewhere"
    )]
    NotLoopback(String),
    #[error("{MOCK_VARIABLE} must be a bare base URL, with no user, path, query or fragment")]
    NotBare,
    #[error("{MOCK_VARIABLE} is set, but chat.token does not begin with {MOCK_TOKEN_PREFIX}")]
    NotAMockToken,
    #[error(
        "the stand-in at {MOCK_VARIABLE} named a gateway that is not ws:// on a loopback IP address: {0}"
    )]
    GatewayNotLocal(String),
    #[error("the REST client for the stand-in could not be built: {0}")]
    Client(String),
}

/// Whether `url` is `scheme` on a loopback IP literal, with no user part.
fn on_loopback(url: &Url, scheme: &str) -> Result<(), Refused> {
    if url.scheme() != scheme {
        return Err(Refused::NotHttp(url.scheme().to_owned()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refused::NotBare);
    }
    let address = match url.host() {
        Some(Host::Ipv4(address)) => IpAddr::V4(address),
        Some(Host::Ipv6(address)) => IpAddr::V6(address),
        Some(Host::Domain(name)) => return Err(Refused::NotLoopback(name.to_owned())),
        None => return Err(Refused::NotLoopback(String::new())),
    };
    if !address.is_loopback() {
        return Err(Refused::NotLoopback(address.to_string()));
    }
    Ok(())
}

impl Endpoint {
    /// Reads the switch from the environment and checks conditions 1 and 2.
    ///
    /// # Errors
    ///
    /// The condition that failed, when the variable is set.
    pub fn from_environment(env: &Environment, token: &str) -> Result<Self, Refused> {
        let Some(raw) = env.get(MOCK_VARIABLE) else {
            return Ok(Self::Discord);
        };
        let url = Url::parse(raw).map_err(|error| Refused::NotAUrl(error.to_string()))?;
        on_loopback(&url, "http")?;
        if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
            return Err(Refused::NotBare);
        }
        if !token.starts_with(MOCK_TOKEN_PREFIX) {
            return Err(Refused::NotAMockToken);
        }
        // The origin is what was checked: scheme, IP literal, port.
        Ok(Self::Mock {
            base: url.origin().ascii_serialization(),
        })
    }

    /// The line errand logs while the stand-in is in force.
    pub fn warning(&self) -> Option<String> {
        match self {
            Self::Discord => None,
            Self::Mock { base } => Some(format!(
                "WARNING: {MOCK_VARIABLE} is set: the chat service is a local stand-in at {base}, not Discord; this is for tests only"
            )),
        }
    }

    /// The client builder: today's for Discord, the guarded one for a
    /// stand-in.
    ///
    /// # Errors
    ///
    /// When the stand-in's REST client cannot be built.
    pub fn builder(&self, token: &str, intents: GatewayIntents) -> Result<ClientBuilder, Refused> {
        match self {
            Self::Discord => Ok(serenity::Client::builder(token, intents)),
            Self::Mock { base } => {
                let client = reqwest012::Client::builder()
                    .no_proxy()
                    .build()
                    .map_err(|error| Refused::Client(error.to_string()))?;
                let http = HttpBuilder::new(token)
                    .proxy(base.as_str())
                    .ratelimiter_disabled(true)
                    .client(client)
                    .build();
                Ok(ClientBuilder::new_with_http(http, intents))
            }
        }
    }

    /// Checks condition 3 once the builder has asked for the gateway URL,
    /// before the client connects.
    ///
    /// # Errors
    ///
    /// When the stand-in named a gateway that is not `ws://` on loopback.
    pub async fn confirm(&self, client: &serenity::Client) -> Result<(), Refused> {
        if *self == Self::Discord {
            return Ok(());
        }
        let ws_url = client.ws_url.lock().await.clone();
        check_gateway(&ws_url)
    }
}

/// Condition 3, on the gateway URL serenity will connect to.
///
/// # Errors
///
/// When it is not `ws://` on a loopback IP literal.
pub fn check_gateway(ws_url: &str) -> Result<(), Refused> {
    let local = Url::parse(ws_url).is_ok_and(|url| on_loopback(&url, "ws").is_ok());
    if local {
        Ok(())
    } else {
        Err(Refused::GatewayNotLocal(ws_url.to_owned()))
    }
}

#[cfg(test)]
mod tests;
