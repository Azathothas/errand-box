//! What the mock's tests share: serenity's REST client pointed at the mock,
//! and a raw gateway socket that shows every frame as it arrives.

#![allow(dead_code, reason = "each test file uses part of this module")]

use std::io::Read;
use std::time::Duration;

use flate2::read::ZlibDecoder;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use serenity::http::{Http, HttpBuilder};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use discord_mock::MockHandle;

/// errand's intents: GUILDS, GUILD_MESSAGES and MESSAGE_CONTENT.
pub const ERRAND_INTENTS: u64 = 33281;

/// serenity's REST client as errand builds it for the mock: the base
/// replaced, the limiter off, no proxy.
pub fn http(mock: &MockHandle) -> Http {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("a REST client");
    HttpBuilder::new(&mock.config().token)
        .proxy(mock.base_url.as_str())
        .ratelimiter_disabled(true)
        .client(client)
        .build()
}

/// A gateway socket, read frame by frame.
pub struct Socket {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

/// A frame as it arrived: its JSON text, inflated when it was compressed.
#[derive(Clone, Debug)]
pub struct Frame {
    pub text: String,
    pub value: Value,
    pub compressed: bool,
}

/// How a socket ended, when it did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ended {
    Closed(Option<u16>),
    Dropped,
}

impl Socket {
    pub async fn open(mock: &MockHandle) -> Self {
        // serenity's spelling once `url` has parsed it: `{base}/?v=10`.
        let (ws, _) = tokio_tungstenite::connect_async(format!("{}/?v=10", mock.ws_url))
            .await
            .expect("the gateway accepts a socket");
        Self { ws }
    }

    /// The next frame, or how the socket ended.
    pub async fn next(&mut self) -> Result<Frame, Ended> {
        loop {
            let message = tokio::time::timeout(Duration::from_secs(10), self.ws.next())
                .await
                .expect("a frame within 10 seconds");
            let (text, compressed) = match message {
                Some(Ok(Message::Text(text))) => (text, false),
                Some(Ok(Message::Binary(bytes))) => {
                    let mut text = String::new();
                    ZlibDecoder::new(&bytes[..])
                        .read_to_string(&mut text)
                        .expect("a zlib payload");
                    (text, true)
                }
                Some(Ok(Message::Close(frame))) => {
                    return Err(Ended::Closed(frame.map(|frame| u16::from(frame.code))));
                }
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => return Err(Ended::Dropped),
            };
            let value = serde_json::from_str(&text).expect("a JSON frame");
            return Ok(Frame {
                text,
                value,
                compressed,
            });
        }
    }

    /// The next frame, which must be one.
    pub async fn frame(&mut self) -> Frame {
        self.next()
            .await
            .expect("a frame, not the end of the socket")
    }

    pub async fn send(&mut self, value: &Value) {
        self.ws
            .send(Message::Text(value.to_string()))
            .await
            .expect("the socket takes a frame");
    }

    /// HELLO, then IDENTIFY with `intents`; returns the HELLO frame.
    pub async fn identify(&mut self, token: &str, intents: u64) -> Frame {
        let hello = self.frame().await;
        self.send(&json!({
            "op": 2,
            "d": {
                "token": format!("Bot {token}"),
                "intents": intents,
                "compress": true,
                "large_threshold": 250,
                "shard": [0, 1],
                "properties": { "os": "linux", "browser": "test", "device": "test" },
            },
        }))
        .await;
        hello
    }

    /// Frames until one is the dispatch `event`; returns them all.
    pub async fn until(&mut self, event: &str) -> Vec<Frame> {
        let mut seen = Vec::new();
        loop {
            let frame = self.frame().await;
            let done = frame.value["t"] == event;
            seen.push(frame);
            if done {
                return seen;
            }
        }
    }
}
