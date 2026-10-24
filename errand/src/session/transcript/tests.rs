use std::io::Write;
use std::sync::{Arc, Mutex};

use super::Transcript;
use crate::log::{LogFields, LogLevel, Logger, Sink};
use crate::session::event::SessionEvent;

/// A temp directory with a transcript in it, and the lines its log caught.
struct Keep {
    _root: tempfile::TempDir,
    path: std::path::PathBuf,
    lines: Arc<Mutex<Vec<(LogLevel, String)>>>,
}

fn keep() -> Keep {
    let root = tempfile::tempdir().expect("a temp directory");
    let path = root.path().join("transcript.jsonl");
    Keep {
        _root: root,
        path,
        lines: Arc::new(Mutex::new(Vec::new())),
    }
}

impl Keep {
    fn transcript(&self) -> Transcript {
        let lines = Arc::clone(&self.lines);
        let sink: Sink =
            Arc::new(move |level, line| lines.lock().unwrap().push((level, line.to_owned())));
        Transcript::new(&self.path, Some(Logger::new(LogFields::new(), sink)))
    }

    fn logged(&self) -> Vec<(LogLevel, String)> {
        self.lines.lock().unwrap().clone()
    }
}

fn post(text: &str) -> SessionEvent {
    SessionEvent::Post {
        text: text.to_owned(),
    }
}

#[test]
fn what_is_appended_comes_back_in_order_with_its_turn() {
    let keep = keep();
    let transcript = keep.transcript();
    transcript.append_at(&post("first"), Some(1), 1_000);
    transcript.append_at(
        &SessionEvent::Activity {
            line: "ran something".to_owned(),
            tool: None,
        },
        Some(2),
        2_000,
    );

    let stored = transcript.read();

    assert_eq!(stored.entries.len(), 2);
    assert_eq!(stored.entries[0].turn, Some(1));
    assert_eq!(stored.entries[0].at, 1_000);
    assert!(matches!(
        &stored.entries[1].entry,
        SessionEvent::Activity { .. }
    ));
    assert_eq!(stored.dropped, 0);
}

#[test]
fn a_session_with_no_transcript_reads_empty_rather_than_failing() {
    let keep = keep();
    let missing = Transcript::new(keep.path.with_extension("nowhere"), None);

    let stored = missing.read();

    assert_eq!(stored.entries.len(), 0);
    assert_eq!(stored.dropped, 0);
}

/// A crash mid-write tears the last line; the rest of the session survives.
#[test]
fn a_torn_line_is_skipped_and_everything_around_it_is_kept() {
    let keep = keep();
    let transcript = keep.transcript();
    transcript.append_at(&post("before"), Some(1), 1_000);
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&keep.path)
        .expect("the transcript opens");
    file.write_all(r#"{"at":2000,"turn":1,"entry":{"call":"po"#.as_bytes())
        .expect("the tear is written");
    file.write_all(b"\n").expect("the tear ends");
    transcript.append_at(&post("after"), Some(1), 3_000);

    let stored = transcript.read();

    assert_eq!(
        stored
            .entries
            .iter()
            .map(|held| held.entry.clone())
            .collect::<Vec<_>>(),
        [post("before"), post("after")]
    );
}

#[test]
fn anything_that_is_not_an_entry_is_ignored() {
    let keep = keep();
    let transcript = keep.transcript();
    std::fs::write(&keep.path, "\"a bare string\"\n{\"no\":\"entry\"}\n[]\n\n")
        .expect("the noise is written");
    transcript.append_at(&post("real"), Some(1), 1_000);

    assert_eq!(transcript.read().entries.len(), 1);
}

#[test]
fn only_the_most_recent_are_read_back_and_the_gap_is_admitted() {
    let keep = keep();
    let transcript = keep.transcript();
    for index in 0..10 {
        transcript.append_at(&post(&format!("line {index}")), Some(1), index);
    }

    let stored = transcript.read_up_to(4);

    assert_eq!(stored.entries.len(), 4);
    assert_eq!(stored.dropped, 6);
    assert_eq!(stored.entries[0].entry, post("line 6"));
}

/// A transcript written before turns were kept still reads.
#[test]
fn an_entry_with_no_turn_is_still_an_entry() {
    let keep = keep();
    let transcript = keep.transcript();
    std::fs::write(
        &keep.path,
        "{\"at\":1,\"entry\":{\"call\":\"post\",\"text\":\"old\"}}\n",
    )
    .expect("the old line is written");

    let stored = transcript.read();

    assert_eq!(stored.entries[0].turn, None);
    assert_eq!(stored.entries[0].entry, post("old"));
}

#[test]
fn a_transcript_that_cannot_be_written_says_so_and_does_not_throw() {
    let keep = keep();
    let blocked = Transcript::new(keep.path.join("nested").join("transcript.jsonl"), {
        let lines = Arc::clone(&keep.lines);
        let sink: Sink =
            Arc::new(move |level, line| lines.lock().unwrap().push((level, line.to_owned())));
        Some(Logger::new(LogFields::new(), sink))
    });

    blocked.append_at(&post("nowhere to go"), Some(1), 1_000);

    let logged = keep.logged();
    assert_eq!(logged.len(), 1);
    assert_eq!(logged[0].0, LogLevel::Warn);
}
