//! A byte relay: accept on one socket, connect to a TCP address, copy both
//! ways. Two uses:
//! - the rootless engine: a guest's broker connection arrives on a unix
//!   socket and leaves for errand's broker on host 127.0.0.1 (E-23);
//! - image builds: the host's egress proxy listens on loopback only, so the
//!   build instance reaches it through a relay on the build bridge (E-10).

use std::io;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

/// How often a relay that keeps failing says so again.
const REPEAT: Duration = Duration::from_secs(10);

/// Says why the relay closed a connection it could not carry: one line at the
/// first failure, then at most one line every `REPEAT`, with the count since
/// the last line. It writes to stderr, never stdout: in the shim, stdout
/// carries errand's agent protocol (T-031).
struct Refusals {
    target: String,
    out: Box<dyn io::Write + Send>,
    last: Option<Instant>,
    since: u64,
}

impl Refusals {
    fn new(target: &str, out: Box<dyn io::Write + Send>) -> Self {
        Refusals {
            target: target.to_owned(),
            out,
            last: None,
            since: 0,
        }
    }

    fn failed(&mut self, error: &io::Error) {
        self.since += 1;
        if self.last.is_some_and(|t| t.elapsed() < REPEAT) {
            return;
        }
        let _ = writeln!(
            self.out,
            "errand-box relay: cannot reach {}: {error}; closed {} connection(s)",
            self.target, self.since
        );
        self.last = Some(Instant::now());
        self.since = 0;
    }
}

fn pump<A, B>(
    a: A,
    b: B,
    close_a: impl Fn(&A) + Send + 'static,
    close_b: impl Fn(&B) + Send + 'static,
) where
    A: io::Read + io::Write + Send + 'static + TryClone,
    B: io::Read + io::Write + Send + 'static + TryClone,
{
    let (Ok(a2), Ok(b2)) = (a.try_clone_box(), b.try_clone_box()) else {
        return;
    };
    thread::spawn(move || {
        let mut a = a;
        let mut b2 = b2;
        let _ = io::copy(&mut a, &mut b2);
        close_b(&b2);
    });
    thread::spawn(move || {
        let mut b = b;
        let mut a2 = a2;
        let _ = io::copy(&mut b, &mut a2);
        close_a(&a2);
    });
}

pub trait TryClone: Sized {
    fn try_clone_box(&self) -> io::Result<Self>;
}

impl TryClone for TcpStream {
    fn try_clone_box(&self) -> io::Result<Self> {
        self.try_clone()
    }
}

impl TryClone for UnixStream {
    fn try_clone_box(&self) -> io::Result<Self> {
        self.try_clone()
    }
}

/// Serve `listener` until the process ends, relaying each connection to
/// `target`. Runs in a background thread.
pub fn unix_to_tcp(path: &Path, target: String) -> io::Result<()> {
    unix_to_tcp_to(path, target, Box::new(io::stderr()))
}

fn unix_to_tcp_to(path: &Path, target: String, out: Box<dyn io::Write + Send>) -> io::Result<()> {
    let _ = std::fs::remove_file(path);
    let l = UnixListener::bind(path)?;
    thread::spawn(move || {
        let mut refusals = Refusals::new(&target, out);
        for c in l.incoming().flatten() {
            match TcpStream::connect(&target) {
                Ok(up) => pump(
                    c,
                    up,
                    |s: &UnixStream| drop(s.shutdown(Shutdown::Write)),
                    |s: &TcpStream| drop(s.shutdown(Shutdown::Write)),
                ),
                Err(e) => refusals.failed(&e),
            }
        }
    });
    Ok(())
}

/// Serve on `listen` (host:port) and relay to `target`, in the foreground.
pub fn tcp_to_tcp(listen: &str, target: &str) -> io::Result<()> {
    let l = TcpListener::bind(listen)?;
    let mut refusals = Refusals::new(target, Box::new(io::stderr()));
    for c in l.incoming().flatten() {
        match TcpStream::connect(target) {
            Ok(up) => pump(
                c,
                up,
                |s: &TcpStream| drop(s.shutdown(Shutdown::Write)),
                |s: &TcpStream| drop(s.shutdown(Shutdown::Write)),
            ),
            Err(e) => refusals.failed(&e),
        }
    }
    Ok(())
}

/// `host:port` of a proxy URL such as `http://user:pw@127.0.0.1:3128/`.
pub fn proxy_authority(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://").unwrap_or(("http", url));
    let rest = rest.split('/').next()?;
    let rest = rest.rsplit_once('@').map(|(_, h)| h).unwrap_or(rest);
    if rest.is_empty() {
        return None;
    }
    // A port-less proxy URL means the scheme's port.
    let has_port = match rest.strip_prefix('[') {
        Some(v6) => v6.split_once(']').is_some_and(|(_, p)| p.starts_with(':')),
        None => rest.contains(':'),
    };
    if has_port {
        Some(rest.to_owned())
    } else {
        Some(format!(
            "{rest}:{}",
            if scheme.eq_ignore_ascii_case("https") {
                443
            } else {
                80
            }
        ))
    }
}

/// `url` with its host and port replaced by `authority`; the scheme, any
/// credentials and the path are kept.
pub fn with_authority(url: &str, authority: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("http", url));
    let (hostpart, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let userinfo = hostpart
        .rsplit_once('@')
        .map(|(u, _)| format!("{u}@"))
        .unwrap_or_default();
    format!("{scheme}://{userinfo}{authority}{path}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    /// A sink the test can read after the relay wrote to it.
    #[derive(Clone, Default)]
    struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl io::Write for Sink {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// A target that refuses is reported once, with its address, and a
    /// second refusal inside the repeat window adds no line (T-031).
    #[test]
    fn a_relay_says_why_it_closed_a_connection() {
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let target = closed.local_addr().unwrap().to_string();
        drop(closed);
        let dir = std::env::temp_dir().join(format!("eb-relay-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("r.sock");
        let sink = Sink::default();
        unix_to_tcp_to(&sock, target.clone(), Box::new(sink.clone())).unwrap();
        for _ in 0..2 {
            let mut c = UnixStream::connect(&sock).unwrap();
            let mut got = Vec::new();
            // The relay closes the connection it cannot carry.
            let _ = c.read_to_end(&mut got);
            assert!(got.is_empty());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while sink.0.lock().unwrap().is_empty() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        let text = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(text.lines().count(), 1, "{text}");
        assert!(text.contains(&format!("cannot reach {target}")), "{text}");
        assert!(text.contains("closed 1 connection(s)"), "{text}");
    }

    #[test]
    fn a_relayed_proxy_keeps_its_credentials() {
        assert_eq!(
            with_authority("http://u:p@127.0.0.1:3128", "10.25.127.1:40000"),
            "http://u:p@10.25.127.1:40000"
        );
        assert_eq!(
            with_authority("http://127.0.0.1:3128/", "10.0.0.1:1"),
            "http://10.0.0.1:1/"
        );
        assert_eq!(
            proxy_authority("http://proxy.example").as_deref(),
            Some("proxy.example:80")
        );
        assert_eq!(
            proxy_authority("https://proxy.example").as_deref(),
            Some("proxy.example:443")
        );
        assert_eq!(
            proxy_authority("http://[::1]:3128").as_deref(),
            Some("[::1]:3128")
        );
        assert_eq!(proxy_authority("http://[::1]").as_deref(), Some("[::1]:80"));
    }

    #[test]
    fn authority_from_proxy_urls() {
        assert_eq!(
            proxy_authority("http://127.0.0.1:45823").as_deref(),
            Some("127.0.0.1:45823")
        );
        assert_eq!(
            proxy_authority("http://u:p@10.0.0.1:3128/").as_deref(),
            Some("10.0.0.1:3128")
        );
        assert_eq!(
            proxy_authority("proxy.example:8080").as_deref(),
            Some("proxy.example:8080")
        );
        assert_eq!(
            proxy_authority("http://noport").as_deref(),
            Some("noport:80")
        );
        assert_eq!(proxy_authority("http://"), None);
    }

    #[test]
    fn unix_socket_reaches_a_tcp_server() {
        let srv = TcpListener::bind("127.0.0.1:0").unwrap();
        let target = srv.local_addr().unwrap().to_string();
        thread::spawn(move || {
            let (mut c, _) = srv.accept().unwrap();
            let mut buf = [0u8; 4];
            c.read_exact(&mut buf).unwrap();
            c.write_all(&buf.map(|b| b.to_ascii_uppercase())).unwrap();
        });
        let sock = std::env::temp_dir().join(format!("eb-relay-{}.sock", std::process::id()));
        unix_to_tcp(&sock, target).unwrap();
        let mut u = UnixStream::connect(&sock).unwrap();
        u.write_all(b"ping").unwrap();
        let mut back = [0u8; 4];
        u.read_exact(&mut back).unwrap();
        assert_eq!(&back, b"PING");
        let _ = std::fs::remove_file(&sock);
    }
}
