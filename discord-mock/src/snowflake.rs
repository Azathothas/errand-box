//! Snowflake ids in Discord's layout, and the timestamps derived from them.
//!
//! A snowflake is the milliseconds since the Discord epoch in the top 42
//! bits, then a 5-bit worker id, a 5-bit process id and a 12-bit increment
//! (`discord/discord-api-docs@6128d99116af developers/reference.mdx`,
//! "Snowflakes"). Ids from one generator rise strictly, so an id sorts in the
//! order it was made, as Discord's do.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// The first millisecond of 2015, in Unix milliseconds: Discord's epoch.
pub const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;

/// The worker and process bits every id of this mock carries.
const WORKER: u64 = 1;
const PROCESS: u64 = 0;

/// Makes rising snowflake ids.
#[derive(Debug, Default)]
pub struct Snowflakes {
    /// The millisecond and the increment of the last id made.
    last: Mutex<(u64, u64)>,
}

impl Snowflakes {
    /// The next id: now, or one past the last id when the clock has not
    /// moved, so two ids are never equal and never fall.
    pub fn next(&self) -> u64 {
        let now = now_ms().saturating_sub(DISCORD_EPOCH_MS);
        let mut last = self.last.lock().expect("the snowflake lock");
        let (ms, increment) = if now > last.0 {
            (now, 0)
        } else if last.1 < 0xfff {
            (last.0, last.1 + 1)
        } else {
            (last.0 + 1, 0)
        };
        *last = (ms, increment);
        (ms << 22) | (WORKER << 17) | (PROCESS << 12) | increment
    }
}

/// The Unix millisecond a snowflake was made at.
pub fn unix_ms_of(id: u64) -> u64 {
    (id >> 22) + DISCORD_EPOCH_MS
}

/// The Unix time now, in milliseconds.
pub fn now_ms() -> u64 {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
}

/// A Unix millisecond as Discord writes a timestamp:
/// `2026-10-24T10:00:00.000000+00:00`.
pub fn iso(unix_ms: u64) -> String {
    let at = i64::try_from(unix_ms)
        .ok()
        .and_then(|ms| jiff::Timestamp::from_millisecond(ms).ok())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH);
    at.strftime("%Y-%m-%dT%H:%M:%S%.6f+00:00").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_rise_and_carry_the_time_they_were_made() {
        let ids = Snowflakes::default();
        let before = now_ms();
        let made: Vec<u64> = (0..5000).map(|_| ids.next()).collect();
        assert!(made.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(unix_ms_of(made[0]) >= before);
        assert!(unix_ms_of(made[0]) <= now_ms() + 2);
    }

    #[test]
    fn a_timestamp_has_discords_shape() {
        assert_eq!(iso(1_420_070_400_000), "2015-01-01T00:00:00.000000+00:00");
        assert_eq!(iso(1_420_070_400_123), "2015-01-01T00:00:00.123000+00:00");
    }
}
