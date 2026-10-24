//! The `discord-mock` binary: it binds, writes its port, serves Discord's
//! routes and the `/_mock/` control, and refuses a seed with an unknown key.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const BINARY: &str = env!("CARGO_BIN_EXE_discord-mock");

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("discord-mock-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

async fn body(response: reqwest::Response) -> Value {
    serde_json::from_slice(&response.bytes().await.expect("a body")).expect("JSON")
}

#[tokio::test]
async fn the_binary_serves_on_the_port_it_writes() {
    let dir = scratch("serve");
    let port_file = dir.join("port");
    let mut child = Command::new(BINARY)
        .args(["--listen", "127.0.0.1:0", "--port-file"])
        .arg(&port_file)
        .stdout(Stdio::null())
        .spawn()
        .expect("the binary starts");
    let started = Instant::now();
    let port = loop {
        if let Ok(text) = std::fs::read_to_string(&port_file)
            && let Ok(port) = text.trim().parse::<u16>()
        {
            break port;
        }
        assert!(started.elapsed() < Duration::from_secs(10), "no port file");
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("a client");
    let gateway = client
        .get(format!("{base}/api/v10/gateway"))
        .header("Authorization", "Bot mock.bot-token")
        .send()
        .await
        .expect("an answer");
    assert_eq!(gateway.status().as_u16(), 200);
    assert!(
        body(gateway).await["url"]
            .as_str()
            .is_some_and(|url| url.starts_with("ws://127.0.0.1:"))
    );
    let posted = client
        .post(format!("{base}/_mock/messages"))
        .header("Content-Type", "application/json")
        .body(json!({ "content": "from outside" }).to_string())
        .send()
        .await
        .expect("an answer");
    assert_eq!(body(posted).await["content"], "from outside");
    let journal = body(
        client
            .get(format!("{base}/_mock/journal"))
            .send()
            .await
            .expect("an answer"),
    )
    .await;
    assert_eq!(journal.as_array().map(Vec::len), Some(1), "{journal}");
    child.kill().expect("the child stops");
    child.wait().expect("the child is reaped");
    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}

#[test]
fn a_seed_with_an_unknown_key_is_refused() {
    let dir = scratch("seed");
    let seed = dir.join("seed.json");
    std::fs::write(&seed, r#"{ "token": "mock.x", "colour": "blue" }"#).expect("a seed");
    let status = Command::new(BINARY)
        .arg("--seed")
        .arg(&seed)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("the binary runs");
    assert_eq!(status.code(), Some(2));
    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}
