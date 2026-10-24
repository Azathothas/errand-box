//! Every request, answer and frame the mock saw, in order, for assertions.
//!
//! A test reads the journal rather than the mock's state, because the
//! journal is what the client did: which call came first, which body it
//! sent, which frame it got. A gap, a route or a field the mock does not
//! implement, is an entry too, and the handle refuses to drop quietly while
//! one is unread.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::watch;

/// One thing that crossed the mock's edge.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    /// A REST or CDN call and the answer it got.
    Rest {
        method: String,
        /// The path as sent, without the query.
        path: String,
        /// The route it matched, in the spec's spelling with `/api/v10`
        /// left on; none when no route matched.
        route: Option<String>,
        query: Option<String>,
        status: u16,
        /// The body as JSON; a multipart upload as its parts' summary.
        request: Value,
        /// The answer's body as JSON, or null when it had none.
        response: Value,
    },
    /// A gateway socket opened, with the query string it asked for.
    Connect { query: String },
    /// A gateway frame the client sent.
    Received { op: u64, payload: Value },
    /// A gateway frame the mock sent, as JSON before any compression.
    Sent { payload: Value, compressed: bool },
    /// The mock ended a gateway socket: with a close code, or by dropping it.
    Closed { code: Option<u16>, reason: String },
    /// Something was asked for that the mock does not implement.
    Gap { what: String },
}

impl Entry {
    /// The gateway frame's event name, for a dispatch the mock sent.
    pub fn sent_event(&self) -> Option<&str> {
        match self {
            Self::Sent { payload, .. } => payload.get("t").and_then(Value::as_str),
            _ => None,
        }
    }

    /// Whether this is a REST call with this method and route.
    pub fn is_call(&self, want_method: &str, want_route: &str) -> bool {
        matches!(self, Self::Rest { method, route: Some(route), .. }
            if method == want_method && route == want_route)
    }
}

/// The ordered record, and a signal each time it grows.
#[derive(Debug)]
pub struct Journal {
    entries: Mutex<Vec<Entry>>,
    grown: watch::Sender<usize>,
}

impl Default for Journal {
    fn default() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            grown: watch::Sender::new(0),
        }
    }
}

impl Journal {
    /// Appends one entry.
    pub fn push(&self, entry: Entry) {
        let len = {
            let mut entries = self.entries.lock().expect("the journal lock");
            entries.push(entry);
            entries.len()
        };
        self.grown.send_replace(len);
    }

    /// Everything so far.
    pub fn entries(&self) -> Vec<Entry> {
        self.entries.lock().expect("the journal lock").clone()
    }

    /// What each gap entry names.
    pub fn gaps(&self) -> Vec<String> {
        self.entries
            .lock()
            .expect("the journal lock")
            .iter()
            .filter_map(|entry| match entry {
                Entry::Gap { what } => Some(what.clone()),
                _ => None,
            })
            .collect()
    }

    /// Waits until `found` answers for the entries so far, no longer than
    /// `limit`. Returns its answer, or an error naming the wait.
    pub async fn wait_for<T>(
        &self,
        limit: Duration,
        mut found: impl FnMut(&[Entry]) -> Option<T>,
    ) -> Result<T, String> {
        let mut grown = self.grown.subscribe();
        let wait = async {
            loop {
                if let Some(answer) = found(&self.entries.lock().expect("the journal lock")) {
                    return answer;
                }
                if grown.changed().await.is_err() {
                    // The sender lives as long as the journal, so this does
                    // not happen while anybody can still be waiting.
                    std::future::pending::<()>().await;
                }
            }
        };
        tokio::time::timeout(limit, wait)
            .await
            .map_err(|_| format!("nothing in the journal matched within {limit:?}"))
    }
}
