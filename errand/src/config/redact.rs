//! Removes secret values from anything about to be logged or posted.
//!
//! Two layers, because either alone leaks. Structural redaction blanks the
//! known secret fields when the effective configuration is rendered. Value
//! redaction scrubs the same strings out of arbitrary text, since an error
//! thrown by a library may carry a token that never passed through the config
//! renderer.

use serde_json::Value;

use crate::config::schema::{Config, SECRET_PATHS};

/// What a secret is replaced with. Fixed so it is greppable in a log.
pub const REDACTION: &str = "[redacted]";

/// Shortest secret worth scrubbing from free text, to avoid mangling prose.
const MIN_SCRUBBABLE_LENGTH: usize = 8;

/// The prefix every broker nonce carries.
///
/// A broker nonce stands in for a provider key in a session's environment, so
/// it is key-shaped and worth hiding wherever a session echoes it. It is minted
/// per run and never written to the config, so there is no value to add to the
/// secret list at startup; this prefix lets it be recognised and scrubbed by
/// shape instead. `serve::provider_nonce` mints nonces with it (T-032).
pub const BROKER_NONCE_PREFIX: &str = "errand-nonce-";

/// Shortest hex body after the prefix that is taken for a nonce rather than
/// prose. A real nonce is 64 hex characters; this still catches a truncated one
/// while leaving a bare mention of the prefix in documentation alone.
const MIN_NONCE_BODY: usize = 16;

/// Renders the effective configuration with every secret field blanked, for
/// logging at startup.
///
/// The fixed paths in [`SECRET_PATHS`] are not the whole of it. An operator
/// may define any number of providers under any names they like, each with a
/// credential of its own, so those are walked rather than named.
pub fn redact_config(config: &Config) -> Value {
    let mut clone =
        serde_json::to_value(config).expect("the resolved configuration always renders as JSON");
    for path in SECRET_PATHS {
        let Some((parent_path, last)) = path.rsplit_once('.') else {
            continue;
        };
        if let Some(Value::Object(fields)) = read_path_mut(&mut clone, parent_path) {
            fields.insert(last.to_owned(), Value::String(REDACTION.to_owned()));
        }
    }

    // A defined provider carries its own credential, and there is no fixed
    // path to name it by: the operator chooses how many providers there are
    // and what they are called. Walked instead, so a second provider's key is
    // blanked as thoroughly as the first one's.
    if let Some(Value::Object(providers)) = read_path_mut(&mut clone, "agent.providers") {
        for definition in providers.values_mut() {
            if let Value::Object(fields) = definition
                && fields.contains_key("credential")
            {
                fields.insert("credential".to_owned(), Value::String(REDACTION.to_owned()));
            }
        }
    }
    clone
}

/// Collects the secret values held by a configuration, for scrubbing text.
pub fn secret_values(config: &Config) -> Vec<String> {
    let mut values = Vec::new();
    let mut hold = |value: Option<&str>| {
        if let Some(value) = value.filter(|value| value.len() >= MIN_SCRUBBABLE_LENGTH) {
            values.push(value.to_owned());
        }
    };
    hold(Some(&config.chat.token));
    if let Some(github) = &config.github {
        hold(Some(&github.token));
    }
    // A defined provider carries its own credential, and there is no fixed
    // path to name it by: the operator chooses how many providers there are
    // and what they are called. Walked instead, so a second provider's key is
    // scrubbed as thoroughly as the first one's.
    for definition in config.agent.providers.values() {
        hold(definition.get("credential").and_then(Value::as_str));
    }
    values
}

/// Scrubs known secret values out of arbitrary text.
///
/// A secret shorter than the minimum is left alone: replacing a short string
/// everywhere it occurs mangles ordinary prose without hiding anything worth
/// hiding.
pub fn redact_text(text: &str, secrets: &[String]) -> String {
    let mut scrubbed = text.to_owned();
    for secret in secrets {
        if secret.len() < MIN_SCRUBBABLE_LENGTH {
            continue;
        }
        scrubbed = scrubbed.replace(secret.as_str(), REDACTION);
    }
    redact_nonces(&scrubbed)
}

/// Scrubs broker nonces by shape: the prefix and the hex body that follows it.
///
/// A nonce is minted per run, so it is not in the secret list, but it is a
/// working credential through the broker until the daemon restarts and is
/// shaped like a key. Judged by the prefix, so a value re-encoded by an agent
/// is missed the same way a verbatim secret is (the layer above says as much).
fn redact_nonces(text: &str) -> String {
    if !text.contains(BROKER_NONCE_PREFIX) {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(BROKER_NONCE_PREFIX) {
        out.push_str(&rest[..at]);
        let after = &rest[at + BROKER_NONCE_PREFIX.len()..];
        let body = after
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(after.len());
        if body >= MIN_NONCE_BODY {
            out.push_str(REDACTION);
            rest = &after[body..];
        } else {
            // A bare mention of the prefix, not a nonce: leave it as it is.
            out.push_str(BROKER_NONCE_PREFIX);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Walks a dotted path to the value it names, when every step is an object.
fn read_path_mut<'a>(source: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    let mut current = source;
    for segment in path.split('.') {
        current = current.as_object_mut()?.get_mut(segment)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests;
