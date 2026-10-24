//! `discord-mock`: the stand-in as a process, for a CI job that runs errand
//! against it from outside.
//!
//! Usage: `discord-mock [--listen 127.0.0.1:0] [--seed FILE] [--port-file FILE]`
//!
//! `--seed` reads the configuration as JSON (the keys of `Config`);
//! `--port-file` receives the REST port once both listeners are bound. The
//! process serves until it is killed. A person's message is injected with
//! `POST /_mock/messages`, a fault with `POST /_mock/fault`, and the journal
//! is read with `GET /_mock/journal`.
//!
//! Exit codes: 0 never (it serves until killed), 2 bad arguments or a port
//! that cannot be bound.

use std::net::SocketAddr;
use std::process::ExitCode;

use discord_mock::{Config, Mock};

fn usage(why: &str) -> ExitCode {
    eprintln!("discord-mock: {why}");
    eprintln!("usage: discord-mock [--listen 127.0.0.1:0] [--seed FILE] [--port-file FILE]");
    ExitCode::from(2)
}

#[tokio::main]
async fn main() -> ExitCode {
    let mut listen = SocketAddr::from(([127, 0, 0, 1], 0));
    let mut config = Config::default();
    let mut port_file = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let Some(value) = args.next() else {
            return usage(&format!("{arg} needs a value"));
        };
        match arg.as_str() {
            "--listen" => match value.parse() {
                Ok(address) => listen = address,
                Err(error) => return usage(&format!("--listen {value}: {error}")),
            },
            "--seed" => {
                let parsed = std::fs::read_to_string(&value)
                    .map_err(|error| error.to_string())
                    .and_then(|text| {
                        serde_json::from_str(&text).map_err(|error| error.to_string())
                    });
                match parsed {
                    Ok(seed) => config = seed,
                    Err(error) => return usage(&format!("--seed {value}: {error}")),
                }
            }
            "--port-file" => port_file = Some(value),
            _ => return usage(&format!("unknown argument {arg}")),
        }
    }
    let mock = match Mock::start_on(config, listen).await {
        Ok(mock) => mock,
        Err(error) => return usage(&format!("cannot listen on {listen}: {error}")),
    };
    println!(
        "discord-mock: REST {} gateway {}",
        mock.base_url, mock.ws_url
    );
    if let Some(path) = port_file {
        let port = mock.base_url.rsplit(':').next().unwrap_or_default();
        if let Err(error) = std::fs::write(&path, format!("{port}\n")) {
            return usage(&format!("--port-file {path}: {error}"));
        }
    }
    std::future::pending::<()>().await;
    ExitCode::SUCCESS
}
