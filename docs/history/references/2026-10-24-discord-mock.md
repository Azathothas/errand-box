# Reference sweep: a Discord stand-in for errand's chat path

Read on 2026-10-24 for one decision: how errand-box tests errand's chat path
end to end without Discord, so that a pass means something against Discord.
The procedure is [`../../methodology/research.md`](../../methodology/research.md)
sections 1, 2, 3 and 8. errand is cited at errand-box `6a098da76ce0` (no
cited file changed since `e7cc0df0abce`, where the reading began), serenity
from the 0.12.5 crate source, a clone as `OWNER/REPO@COMMIT path:line`.
"Estimate" marks every number that was not measured.

**The questions, each with a wrong answer.** (1) Which REST routes and
gateway events does errand reach through serenity 0.12.5? (2) Can serenity
reach a non-Discord host with a change in errand only, and with what guard?
(3) Does an existing Discord reimplementation or mock serve that surface well
enough to run instead of writing one? (4) Which route is cheapest at the
fidelity the operator asked for? (5) What breaks each route?

**What would make me choose a different route**, written before the
candidates were read: (a) an existing server that serves serenity's whole
surface for errand, with a licence that lets us run and patch it, and no
runtime beyond what the test hosts have, would make "run it" win; (b) a
surface larger than about 40 operations would make a focused mock too costly;
(c) a serenity limit that forces a patched serenity would favour contract
tests against Discord. None of the three came true; section "Q3" and "Q2"
say how each came out.

| a reader with | reads |
| --- | --- |
| two minutes | "Recommendation" |
| ten minutes | the two lists below, then "Recommendation" |
| the implementation to do | "Implementation plan", then "Q1" and "Q2" |
| a reason to distrust this page | "Q2", the provenance table, then the probe commands |

## What this sweep did NOT establish

- **Nothing was built or run against serenity**, because another session
  held the machine's one build. No mock exists, no patch was compiled, and
  every claim about serenity's behaviour is from reading its source.
- **No real Discord traffic was captured**: no bot token in this session.
  Gateway shapes come from Discord's docs and two client models.
- **Spacebar and fauxcord were not run.** Their costs come from manifests
  and code; no start-up time, size or memory was measured.
- **The spec probe ran on one machine** (this Ubuntu 24.04.5 container,
  Python 3.13.17, `jsonschema` 4.26.0), not on the packaged 4.10.3 the plan
  names. Its two scripts stay in session scratch, because this unit was
  limited to one file; test 4 replaces them.
- **Trackers were read by keyword and by number**, not every comment;
  discussions were not fetched (GraphQL only); serenity's tracker was read
  through search on four topics. Search went through the read-only proxy of
  [`../../hosted-sessions.md`](../../hosted-sessions.md) section 7, because
  the direct route refused it. A project no search named is not here.
- **This page has no earlier revision**, so there is no error rate to quote.
  Assume more errors remain.

## The claims I am least sure of

1. **That no real Discord bot token can begin with `mock.`.** The guard rests
   on it. It is from the token's known shape (a base64 user id, a dot, two
   more segments), not from a Discord document read here.
2. **That Discord accepts `Bot ` in front of the IDENTIFY token.** serenity
   sends `http.token()` there, which carries the prefix
   (`serenity-0.12.5/src/gateway/bridge/shard_queuer.rs:167`,
   `http/client.rs:169-178`). It works against Discord in production, so the
   inference is strong; it was not read in a document.
3. **The three events Discord sends after errand creates a thread**
   (THREAD_CREATE, MESSAGE_UPDATE, THREAD_MEMBER_UPDATE). The first two are
   documented (`discord/discord-api-docs@6128d99116af
   developers/resources/channel.mdx:612`); the third is inferred from
   `developers/events/gateway-events.mdx:637`. A capture settles it.
4. **The mock size, 2,000 to 3,500 non-test lines of Rust**, an estimate:
   fauxcord's wider gateway is 1,218 non-test TypeScript lines (measured).
5. **serenity #3575** (op 1 with `"s":null` fails in 0.12.5) is open; my
   reading of `model/utils.rs:73-79` maps null to `None`. A mock test settles
   it.

## Recommendation

**Write a focused Rust mock in this workspace (route b), shaped and checked
against Discord's own spec, and point serenity at it with one guarded change
in errand.** Add a contract job against the operator's test bot later (route
d), outbound only. Use one real capture for gateway fixtures (route c, once).
Do not run Spacebar or fauxcord.

1. **The surface is small and fixed.** errand reaches 15 REST operations, one
   CDN fetch, 4 client opcodes, 6 server opcodes and 11 dispatch types
   ("Q1"). That is far below the 40-operation line set above.
2. **serenity reaches a mock with no serenity patch.** `HttpBuilder::proxy`
   plus `ratelimiter_disabled(true)` moves REST
   (`http/request.rs:76-79`), and the gateway URL comes from `GET /gateway`,
   which the mock answers with `ws://` (`client/mod.rs:364-370`,
   `gateway/shard.rs:766-775`). The price is serenity's rate limiter, which
   must be off on this route ("Q2").
3. **Nothing that exists fits.** Spacebar needs Node and Postgres, answers
   every RESUME with INVALID_SESSION, and leaves the deferred interaction as
   a TODO. fauxcord is the closest and has no licence. The in-process fakes
   serve Python and JavaScript libraries only ("Q3").
4. **Fidelity comes from oracles, not from size.** REST answers are checked
   against `discord/discord-api-spec`. Discord ruled gateway schemas out of
   that repository (its #19), so gateway payloads are checked by two
   independent deserializers, then by a real capture. The spec is stricter
   than serenity: a message serenity accepts fails it on three fields ("Q4").

## Implementation plan

Write a TODO entry for this first ([`../../methodology/authoring.md`](../../methodology/authoring.md)),
in the `errand` category, linked from T-015.

### Where it lives and what it depends on

A new workspace member, `discord-mock/` (crate `discord-mock`,
`publish = false`, licence 0BSD like the root crate), with a library for
in-process tests and a `discord-mock` binary for the live tests. Add it to
`members` in the root `Cargo.toml`. It is never a dependency of `errand` or
`errand-box`, only a dev-dependency of `errand`.

| dependency | version | in our lock | why |
| --- | --- | --- | --- |
| `tokio` | 1.53.1 | yes | runtime; features `rt-multi-thread`, `macros`, `net`, `sync`, `time` |
| `axum` | 0.8.9 | yes | REST router. Add feature `multipart`: it adds `multer` 3.1.0 and `spin`, the only 2 new crate names (measured) |
| `tokio-tungstenite` | 0.21.0 | yes | gateway server with `accept_hdr_async` (feature `handshake` is on); the same version serenity uses |
| `futures-util` | 0.3.34 | yes | socket split |
| `serde`, `serde_json` | 1.0.229, 1.0.154 | yes | payloads |
| `flate2` | 1.1.10 | yes | zlib payload compression for `compress: true` |
| `getrandom` | 0.4.3 | yes | session ids, interaction tokens, the signed-URL `hm` |
| `jiff` | 0.2.37 | yes | ISO-8601 timestamps |
| `http`, `url`, `percent-encoding` | 1.5.0, 2.5.8, 2.3.2 | yes | routing, emoji path segments |
| dev: `serenity` | 0.12.5, errand's features | yes | the client under test |
| dev: `twilight-model` | 0.17.1 | no | second, independent gateway deserializer: 6 new crate names (measured) |

Refused: the Rust `jsonschema` 0.60.1 (39 new crate names with default
features off, measured) and `axum`'s `ws` feature (it wants
`tokio-tungstenite` 0.29, a second copy).

### Module layout

```text
discord-mock/
  Cargo.toml
  spec/errand-surface.json   trimmed discord-api-spec, generated, never edited
  spec/LICENSE-discord-api-spec
  src/lib.rs        Mock::start(Config) -> MockHandle { base_url, ws_url, control }
  src/main.rs       --listen 127.0.0.1:0 --seed FILE --port-file FILE
  src/state.rs      the one store; every mutation returns the dispatches it causes
  src/snowflake.rs  Discord epoch 1420070400000, timestamp bits, per-process increment
  src/rest/         auth, rate-limit headers, the 15 operations, loud gaps
  src/cdn.rs        attachment bytes, signed URLs (ex, is, hm), expiry
  src/gateway/      HELLO, IDENTIFY, READY, heartbeats, seq, replay, RESUME, faults, intents
  src/control.rs    test control: seed, inject, archive, expire, drop, journal
  src/journal.rs    every request, response and frame, for assertions
scripts/discord-spec.sh  regenerate the snapshot; --drift compares upstream
```

Two rules, adopted from SimCord (`SilentHacks/simcord@69bb92dcb458
docs/architecture.md:31-46`): **one store, two projections**, so a REST
write and its gateway event cannot disagree; and **loud gaps**, so an
unimplemented route or an unknown body field answers 501, is journalled, and
fails the test at teardown (`src/simcord/http/router.py:24-60`).

### What to implement first

In this order; each step ends with a test.

1. `GET /api/v10/gateway` answering `{"url":"ws://127.0.0.1:PORT"}`, the
   socket with HELLO (`heartbeat_interval` 41250 by default, settable),
   IDENTIFY, READY (`v` 10, `session_id`, `resume_gateway_url`, `user`,
   `application.id`, unavailable guild), GUILD_CREATE, HEARTBEAT and ACK,
   and `GET /channels/{id}` for a text channel and a thread.
2. Inject a person's MESSAGE_CREATE in the served channel. The bot's own
   POSTs are echoed as MESSAGE_CREATE too, as Discord does.
   `POST /channels/{c}/messages/{m}/threads` gives the thread the message's
   id, refuses a second thread with 400 code 160004, and dispatches
   THREAD_CREATE and MESSAGE_UPDATE.
3. Edit, delete, get, react and unreact, typing (204), and
   `PATCH /channels/{thread}` with `archived`, which dispatches THREAD_UPDATE.
   An edit or a reaction in an archived thread answers 400 code 50083; a new
   message unarchives it (`discord/discord-api-docs@6128d99116af
   developers/topics/threads.mdx:59`, `:196-203`).
4. Multipart uploads (`files[N]` plus `payload_json`), served back at
   `http://127.0.0.1:PORT/attachments/C/A/NAME?ex=..&is=..&hm=..`, 404 when
   expired or forged; injected user messages carry attachments the same way.
5. `PUT .../guilds/{g}/commands`, an injected INTERACTION_CREATE (type 2),
   the callback (types 4 and 5, flags 64, 204), `PATCH .../@original` and the
   follow-up POST. A token older than 15 minutes, or no callback within 3
   seconds, invalidates it (`developers/interactions/receiving-and-responding.mdx:351`).
6. Faults: drop the socket (serenity resumes), op 7, op 9 with `true` and
   `false`, close 4004 and 4014, a withheld ACK, and the 120 sends per 60
   seconds limit with close 4008 (`developers/events/gateway.mdx:509`).
7. Rate limits, for route A2 only ("Q2"). Under A1 the mock sends the
   `X-RateLimit-*` headers on every answer but never a 429.

errand's intents are 33281 (`errand/src/chat/gateway.rs:243-247`). The mock
filters dispatches by intent, so TYPING_START and MESSAGE_REACTION_ADD never
reach errand, as on Discord; without the app's MESSAGE_CONTENT flag it closes
with 4014, and errand exits "refused" (`errand/src/chat/gateway.rs:84-93`).

### The errand patch

**File:** a new `errand/src/chat/endpoint.rs`, called from
`errand/src/serve.rs:937`, where `serenity::Client::builder` is called today.

**Switch:** the environment variable `ERRAND_DISCORD_MOCK_URL`, read from the
environment map errand already builds (`errand/src/serve.rs:1483`). It is not
a config key: errand-box's `setup` and `up` must never write it, and
`errand/src/config/validate.rs:41-48` stays unchanged. The live test sets it
on the command it starts.

**When set**, build
`HttpBuilder::new(token).proxy(URL).ratelimiter_disabled(true).client(C)`,
with `C` a reqwest 0.12 client built with `.no_proxy()`, and pass it to
`ClientBuilder::new_with_http` (`serenity-0.12.5/src/client/mod.rs:116`).
Add `reqwest012 = { package = "reqwest", version = "=0.12.28", default-features = false, features = ["rustls-tls"] }`
to `errand/Cargo.toml`; serenity already locks that version and feature, so
no crate is added. `.no_proxy()` is needed because hyper-util has no
loopback exception (`hyper-util-0.1.20/src/client/proxy/matcher.rs:122-125`);
the supervisor's `NO_PROXY` holds 127.0.0.1 (`src/up.rs:410-415`), a
developer's shell may not. **When unset**, the code path is today's.

**The guard.** The override is honoured only when all three hold; otherwise
errand logs one line and exits with its "refused" code, before any request:

1. the URL is `http://` and its host is an IP literal in 127.0.0.0/8 or `::1`
   (no host name, so no resolver is trusted);
2. `chat.token` begins with `mock.` (least-sure claim 1);
3. after the builder resolves, `client.ws_url` is a `ws://` URL on a loopback
   literal (`serenity-0.12.5/src/client/mod.rs:623`). This closes serenity's
   silent fallback to `wss://gateway.discord.gg` when `GET /gateway` fails
   (`client/mod.rs:364-370`).

What must never happen in production: a real token sent anywhere but
discord.com, a mock run that reaches discord.com, and the override set by
errand-box's own files. Condition 2, condition 3 and the "never write it"
rule close these in that order. errand logs one `WARNING:` line at start
while the override is in force. Since `1d184d3c06a2` (T-028) errand ships
inside every errand-box release, so the guard ships there too.

### The first five tests

1. **The guard can fail** (`errand/src/chat/endpoint.rs` unit tests). A
   non-loopback host, a host name, `https://`, and a token without `mock.`
   are each refused; a correct pair is accepted. Plant the defect: remove
   condition 2 and confirm the test fails, exit code read unpiped.
2. **The handshake is serenity's** (`discord-mock` tests, serenity as
   dev-dependency, built through the same `endpoint` function). The journal
   shows one REST call before the socket, `GET /api/v10/gateway`; the socket
   query is `v=10`; IDENTIFY has `intents` 33281, `compress` true and the
   configured token; READY carries `s` 1; afterwards
   `http.application_id()` equals the mock's application id.
3. **A message opens a thread** (in `errand`, with the `FakeSandbox` of
   `errand/src/daemon/tests.rs:171` and the real `ChatThreadFactory`). Inject
   a person's message; the journal shows `GET /channels/{channel}`, then
   `POST .../messages/{m}/threads` with `auto_archive_duration` 1440, a
   thread whose id equals `m`, THREAD_CREATE, then a POST in the thread with
   `flags` 4 and `allowed_mentions.parse` `["users"]`.
4. **Every REST answer matches the spec.** Drive the 15 operations; write
   each response body to the journal corpus; `scripts/discord-spec.sh
   --check` validates the corpus against `spec/errand-surface.json` with
   `python3 -I` and `python3-jsonschema` (apt, 4.10.3 on Ubuntu 24.04), as a
   gate step named `discord-contract`. A skip needs a reason naming a spec
   defect, as fauxcord's list does (`tomacheese/fauxcord@210824790681
   spec/skip.ts:1-53`). Plant the defect: drop `flags` from the message
   answer; the step must exit 1.
5. **Every dispatch parses in two clients, in order.** For each of the 11
   dispatch types, serenity's `GatewayEvent` and twilight-model's
   deserializer both accept the payload without falling to `Unknown`
   (`serenity-0.12.5/src/model/event.rs:1425-1426` would swallow a bad
   shape); `s` rises by one per dispatch; after a dropped socket the RESUME
   replays exactly the missed dispatches, then RESUMED.

Next, in order: archived-thread semantics (implementation step 3), the CDN
expiry, the interaction flow, a withheld ACK leading to a new IDENTIFY no
sooner than 5 seconds (`gateway/bridge/shard_runner.rs:112-117`,
`shard_queuer.rs:134-144`), 4014, and serenity #3575.

### How it stays close to Discord

- **Pin the spec.** `spec/errand-surface.json` is generated by
  `scripts/discord-spec.sh` from `discord/discord-api-spec` at a pinned
  commit, starting with `bf93257f85d6`: the 15 operations and the 250
  components they reach, 164,655 bytes minified against 612,927 for the
  whole spec (measured). The full file's SHA-256 is a pin in `pins.toml`. The spec is
  MIT; keep its licence beside the snapshot.
- **Watch for drift.** Add a check to `scripts/watch-upstreams.sh` (daily,
  `.github/workflows/watchdog.yml`) that regenerates the snapshot from
  upstream and fails when the 15 operations changed. fauxcord runs this
  weekly (`tomacheese/fauxcord@210824790681 .github/workflows/spec-drift.yml:22`);
  it opened six drift reports from 2026-06-29 to 2026-10-19.
- **Two gateway oracles, then a third.** serenity's model and twilight-model
  on every run. When the operator's T-015 bot exists, relay one session,
  scrub ids and tokens, and commit each dispatch type's keys and JSON types
  as fixtures for test 5 to compare against.
- **Pin Discord's docs for semantics.** Each coded rule (thread id equals
  message id, 50083, 160004, unarchive on message, the interaction token's
  15 minutes, 120 sends per minute) cites `discord/discord-api-docs` at a
  commit in a code comment, and has a test.
- **Loud gaps at teardown**, so a new errand call site that reaches an
  unimplemented route fails. **A serenity bump re-runs tests 2 and 5 first.**

### What it does not unblock

T-015's prove also needs an agent that edits a file and runs Docker, so a
model gateway errand-box can configure (T-015's route B, its own entry). The
mock covers the chat half. On test hosts the binary needs a home; T-028 chose
one binary for errand, but this is test code, so the recommendation is a
test-only release asset that `setup` and `bootstrap.sh` never install. That
is the operator's call.

## Evidence

### Q1. The surface errand reaches

**REST** (every call adds `Authorization: Bot TOKEN` and a fixed
`User-Agent`, `serenity-0.12.5/src/http/request.rs:92-94`,
`constants.rs:27-31`):

| # | operation | errand call site | serenity | notes |
| --- | --- | --- | --- | --- |
| 1 | `GET /gateway` | `serve.rs:937` | `client/mod.rs:364` | not `/gateway/bot`: that is `start_autosharded` only (`:717`); errand calls `start` (`serve.rs:959`) |
| 2 | `GET /channels/{id}` | `chat/gateway.rs:474`, `chat/threads.rs:835`, `serve.rs:1302` | `model/channel/channel_id.rs:395-420` | always REST: the cache path needs `temp_cache`, which errand does not enable |
| 3 | `POST /channels/{id}/messages` | `chat/threads.rs:159`, `:822`; `serve.rs:1547-1583`, `:1662` | `http/client.rs:4744-4772` | JSON, or multipart `files[N]` plus `payload_json` (`http/multipart.rs:38-61`); `flags` 4, `allowed_mentions`, `message_reference` on replies |
| 4 | `GET .../messages/{m}` | `chat/threads.rs:205`; `serve.rs:1525`, `:1653` | `http/client.rs:4307` | any error reads as "gone" |
| 5 | `PATCH .../messages/{m}` | `chat/threads.rs:174-180` | `http/client.rs:1957-1987` | the activity block, `flags` 4 |
| 6 | `DELETE .../messages/{m}` | `chat/threads.rs:193` | `http/client.rs:1222` | the waiting message |
| 7 | `PUT .../reactions/{e}/@me` | `chat/threads.rs:222-224` | `http/client.rs:842-861` | emoji percent-encoded (`model/channel/reaction.rs:366-379`) |
| 8 | `DELETE .../reactions/{e}/@me` | `chat/threads.rs:237-239` | `http/client.rs:1386` | replaces the outcome reaction |
| 9 | `PATCH /channels/{id}` | `chat/threads.rs:248-250`, `:849-851` | `http/client.rs:2278` | `{"archived": bool}` |
| 10 | `POST /channels/{id}/typing` | `chat/threads.rs:263`, every 8 s (`:42`) | `http/client.rs:350` | serenity expects 204 |
| 11 | `POST .../messages/{m}/threads` | `chat/threads.rs:800-806`; `serve.rs:1573-1579` | `http/client.rs:411-432` | `auto_archive_duration` 1440 |
| 12 | `PUT /applications/{a}/guilds/{g}/commands` | `chat/commands.rs:242` | `model/guild/guild_id.rs:1574` | needs the application id from READY (`gateway/shard.rs:292-294`) |
| 13 | `POST /interactions/{i}/{t}/callback` | `chat/commands.rs:266-275` (type 4), `:288-295` (type 5) | `http/client.rs:739-769` | flags 64; serenity expects 204 |
| 14 | `PATCH /webhooks/{a}/{t}/messages/@original` | `chat/commands.rs:319-321` | `http/client.rs:2104-2132` | |
| 15 | `POST /webhooks/{a}/{t}` | `chat/commands.rs:323-330` | `http/client.rs:539-567` | ephemeral follow-ups |
| 16 | `GET` the attachment URL | `session/session.rs:2700-2733` | none: reqwest 0.13 | three tries on transport errors, none on a status |

**Rate-limit headers.** Only serenity's `Ratelimiter` reads them
(`http/ratelimiting.rs:337-359`): `x-ratelimit-limit`, `-remaining`,
`-reset-after` (seconds), `-reset` only in absolute mode, `x-ratelimit-global`
(`:218`), and on 429 the `Retry-After` header, not the body's `retry_after`
(`:361-379`).

**Gateway.** The URL is `{url}?v=10`, no `encoding`, no transport
compression (`gateway/shard.rs:766-775`). IDENTIFY sends `compress: true`
(`gateway/ws.rs:275`), so the client inflates any binary frame by itself
(`ws.rs:126-143`). Client opcodes: 1, 2, 3 (presence, from
`chat/gateway.rs:492-498` and `serve.rs:631-636`, every 60 s when usage sources exist), 6. Server
opcodes: 0, 1, 7, 9, 10, 11 (`model/event.rs:1173-1209`). Close codes errand
treats as permanent: 4004, 4013, 4014 (`chat/gateway.rs:84-93`).

| dispatch | who needs it | citation |
| --- | --- | --- |
| READY | errand (bot id, ready), serenity (session, app id) | `chat/gateway.rs:464-468`; `gateway/shard.rs:286-295`; fields `model/gateway.rs:358-374` |
| GUILD_CREATE | serenity's cache; Discord sends it after READY | `client/dispatch.rs:168-190` |
| MESSAGE_CREATE | errand, the inbound path | `chat/gateway.rs:258-292`; bot rules `chat/inbound.rs:187-209` |
| MESSAGE_DELETE | errand, withdrawals | `chat/gateway.rs:352-390` |
| THREAD_UPDATE | errand, archive from outside | `chat/gateway.rs:392-409` |
| THREAD_DELETE | errand | `chat/gateway.rs:411-421` |
| INTERACTION_CREATE | errand, slash commands; not intent-gated | `chat/gateway.rs:294-350` |
| RESUMED | serenity's stage | `gateway/shard.rs:296-303` |
| THREAD_CREATE, MESSAGE_UPDATE, THREAD_MEMBER_UPDATE | Discord sends them after errand's own thread create | least-sure claim 3 |

A payload serenity cannot parse is logged and dropped, not fatal
(`gateway/shard.rs:467-470`); a test that waits for an event is the only
detector.

### Q2. Pointing serenity at something else

| part | answer | citation |
| --- | --- | --- |
| a REST base override | `HttpBuilder::proxy` replaces `https://discord.com` in each URL | `http/client.rs:115-132`, `http/request.rs:76-79` |
| with the rate limiter on | **ignored**: the limiter builds with proxy `None` | `http/ratelimiting.rs:199`; honoured only at `http/client.rs:5218-5219` |
| maintainers' position | the proxy is for twilight-http-proxy, with the limiter off | `http/client.rs:103-127`; serenity #1739, member comment of 2022-02-15 |
| the gateway URL | from `GET /gateway`; on failure, `wss://gateway.discord.gg` | `client/mod.rs:364-370` |
| `ws://` | accepted: plain TCP, no TLS | `tokio-tungstenite-0.21.0/src/tls.rs:89` |
| a test CA for `wss://` | **impossible** without a patch: the root store is webpki-roots only | `tls.rs:95-106`; serenity passes no connector (`gateway/ws.rs:107-116`) |
| a test CA for REST | possible: a custom reqwest 0.12 client | `HttpBuilder::client` (`http/client.rs:93-96`); `reqwest-0.12.28/src/async_impl/client.rs:1854`, `:2270` |
| an HTTP proxy for the socket | none; direct TCP | `tokio-tungstenite-0.21.0/src/connect.rs:55-73`; serenity #2479 open, PRs #3449 and #3590 open |
| `resume_gateway_url` | not used in 0.12.5: resume reconnects to `ws_url` | `gateway/shard.rs:729-742`; serenity PR #2832 merged into `next` only |

Two patches were costed. **A1** (recommended): proxy, limiter off, a
no-proxy client; about 100 lines plus tests (estimate), no new crate. **A2**
keeps the limiter and the `https://discord.com` URLs: a reqwest 0.12 client
resolves `discord.com` to the mock and trusts a test CA, and the mock serves
TLS (`rustls` 0.23.45 and `tokio-rustls` 0.26.5 are locked) with certificates
from `rcgen` (18 new crate names, measured) or `openssl`. Only A2 tests
errand under real 429s. The socket is `ws://` on both.

**This reverses [T-015](../../../TODO/sandbox.md)'s route C** ("refuted").
The roots are compiled in, as T-015 read, but errand is ours now (T-028) and
`ws://` plus the REST override need no TLS. T-015's inferred fact, that the
socket ignores proxies, is confirmed above. The brief's `GET /gateway/bot` is
wrong for errand; fauxcord's harness comment repeats it
(`tomacheese/fauxcord@210824790681 compat/rust-serenity/src/main.rs:212-215`).

### Q3. What exists

| reference | language, licence | last commit | serves errand's surface | fidelity | CI cost |
| --- | --- | --- | --- | --- | --- |
| `spacebarchat/server` | TypeScript, AGPL-3.0 | 2026-10-19 | most routes; deferred interaction is a TODO (`src/api/routes/interactions/#interaction_id/#interaction_token/callback.ts:119-121`) | RESUME always answers op 9 `false` (`src/gateway/opcodes/Resume.ts:22-27`); READY `v` 9 (`Identify.ts:742`); `X-RateLimit-Reset` in milliseconds (`src/api/middlewares/RateLimit.ts:83`) | Node, Postgres only (`src/database/Database.ts:38-46`; migrations for postgres alone), a bot account to create |
| `tomacheese/fauxcord` | TypeScript, **no licence** | 2026-10-23 | the closest: messages, threads, reactions, typing, commands, interactions, `@original`, injection of user messages and interactions (`src/routes/test.ts:125-157`) | resume with replay (`src/gateway/server.ts:364-389`); thread id equals message id (`src/routes/channel-threads.ts:107`); fixed dummy rate-limit headers, never 429 (`src/middleware/rate-limit.ts:33-34`); no payload compression; REST validated against the spec | Node 22, pnpm, `better-sqlite3` native build, or Docker |
| `SilentHacks/simcord` | Python, MIT | 2026-10-22 | discord.py only, in process | "one store, two projections", loud gaps, deterministic snowflakes (`docs/architecture.md:31-64`) | not applicable to Rust |
| `CraftSpider/dpytest` | Python, MIT | 2026-01-13 | discord.py only; no threads (#123 open since 2023) and no interactions (#125 open since 2023) | low | not applicable |
| `Gelbpunkt/mock-gateway` | Rust, AGPL-3.0 | 2023-02-14 | gateway only, "heavy WIP" (README:3), 913 lines, no tests | scripted faults: heartbeat timeouts, failing resumes | small; stale |
| `twilight-rs/http-proxy` | Rust, ISC | 2026-05-19 | forwards to Discord; no gateway, no state | real Discord behind it | needs a real token |
| `glideapps/fake-discord` | TypeScript, no licence | 2026-03-02 | REST and interactions over HTTP; no gateway | per-tenant isolation by token | Cloudflare D1 |
| `leonlarsson/discord-mocker` | TypeScript, no licence | 2026-08-23 | discord.js with a browser UI | not measured | Node |
| `suneettipirneni/discord-mock-server`, `pocketcord/pocketcord`, `minicord-dev/minicord`, `tophcodes/blurplejs` (archived) | TypeScript and Python | 2022 | small and stale; minicord has no gateway | low | not chosen |
| `mavolin/dismock` | Go, MIT | 2024-07-06 | a client-side mock for arikawa, not a server | none for us | not applicable |
| `discord/discord-api-spec` | JSON, MIT | 2026-10-20 | all 15 operations (153 paths, 540 schemas) | REST only; gateway schemas ruled out of scope (#19, maintainer comment) | a file |

**What the trackers say.** Spacebar breaks bot libraries on missing keys
again and again (#1507 discord.js, #1572 discord.py, #1593, #1516); its
resume-with-replay PR #1517 was closed unmerged, as the code at
`190f6310519b` shows. fauxcord's 384 items since 2026-06-07 are 284 from
Renovate, 94 from two people and 6 drift reports. serenity members wrote in
2021 that Discord cannot be mocked reliably (#1247). The spec's #24 (the 429
shape) is open, yet `RatelimitedResponse` at `bf93257f85d6` has
`retry_after` and `global`: the file is newer than the report. Its #23 is
open, so the mock's buckets cannot come from the spec.

**Verdicts.** discord-api-spec: **adopt**, the oracle. fauxcord: **adopt
ideas** (the contract test with a reasoned skip list, drift scoped to
implemented operations, the injection API), not code. SimCord: **adopt** two
rules. Gelbpunkt: **adopt the idea** of scripted faults. Spacebar:
**refused**. twilight http-proxy: **confirms** the limiter-off design. The
rest: **refused**.

### Q4. The routes

| route | verdict | how close it can be | how it stays close |
| --- | --- | --- | --- |
| (a) run an existing server | refused | Spacebar: no resume, no deferred interaction; fauxcord: close, but unlicensed and Node | their maintainers' pace, not ours |
| (b) focused Rust mock | **recommended** | REST checked against the spec; gateway by two deserializers and a capture; Discord semantics coded with citations; serenity's limiter off under A1 | snapshot plus daily drift check, contract step, loud gaps |
| (c) record and replay | refused as the test; adopted once for fixtures | exact bytes, for one moment | it does not: any change in errand's order re-records |
| (d) contract tests against Discord | adopt later, alongside (b) | real, for the outbound half | needs the operator's bot; see "Q5" |

**The probe** (session scratch, conditions above). It validated four
answers against the spec at `bf93257f85d6`, on the full file and on the
trimmed one, with the same verdicts both times:

```sh
python3 -I spec_probe.py specs/openapi.json
python3 -I trim_probe.py specs/openapi.json errand-surface.json
```

| answer | result |
| --- | --- |
| a message with only the fields serenity requires (`serenity-0.12.5/src/model/channel/message.rs:36`) | FAIL: `flags`, `components`, `author.primary_guild` required |
| the same with `flags` and `components` | FAIL: `author.primary_guild` required |
| a thread from `POST .../threads` (201) | PASS |
| `GET /gateway` with `ws://127.0.0.1:41000` | PASS |

So a mock tuned until serenity is satisfied is not yet Discord-shaped, and
the contract step catches the difference. serenity has no
`deny_unknown_fields` anywhere in `src/` (counted: 0), so a spec-complete
answer cannot break it.

### Q5. What breaks each route

**(b), the recommendation:**

- **Spec drift.** The spec is regenerated often ("Automated spec update" at
  `bf93257f85d6`); the drift check turns that into a failing check. Empty
  `enum` arrays (#60, open) can trip a strict validator; none is on our 15
  operations.
- **Gateway shapes have no machine-readable source.** Two deserializers can
  agree on a wrong shape; only a capture breaks the tie.
- **serenity's limiter is off under A1.** A 429 becomes an error in errand
  where Discord plus serenity would wait and retry, so A1 tests send none.
  Route A2 re-opens this.
- **Timing.** A re-IDENTIFY waits 5 seconds after the last
  (`shard_queuer.rs:35`, `:134-144`); a withheld ACK restarts, it does not
  resume (`shard_runner.rs:112-117`); a read waits up to 500 ms
  (`gateway/ws.rs:102`). The HELLO interval must be settable.
- **Attachment URLs are loopback HTTP**, fetched by errand's reqwest 0.13
  client, which honours `HTTP_PROXY` (`session.rs:2701`). The supervisor's
  `NO_PROXY` covers loopback; an in-process test injects `fetch_attachment`,
  because errand forbids `unsafe` and cannot set the environment in tests.
- **Thread and permission semantics** exist only where coded: 50083, 160004,
  unarchive on message, and a per-channel permission set that answers 50013.
- **The cache.** errand reads no channel from it (Q1 row 2), so a thin
  GUILD_CREATE breaks nothing today; a future call through `Context` would.

**(a)** breaks on runtime (Node, Postgres or Docker on hosts with no init and
a proxy), on resume (Spacebar), on licence (fauxcord), and on bot-library
regressions the trackers show. **(c)** breaks on secrets in recordings (the
IDENTIFY token, interaction tokens), on fresh snowflakes and timestamps per
run, on signed CDN URLs that expire (`developers/reference.mdx:415-424`), and
on order: any change in errand invalidates the tape. **(d)** breaks on three
things: a person's message cannot be automated, because a bot's message in
the channel is ignored (`chat/inbound.rs:187-209`) and a scripted user
account is a self-bot; fork pull requests get no secrets; and on hosts
behind a proxy the socket cannot reach `gateway.discord.gg` (Q2).

### Filed elsewhere

- `errand/src/chat/gateway.rs:96-99` and `:471-472` say "from the cache,
  with a REST fallback"; without `temp_cache` it is always REST, one
  `GET /channels/{id}` per MESSAGE_CREATE and MESSAGE_DELETE. An errand entry.
- errand marks the connection down only on `Disconnected`
  (`chat/gateway.rs:434-441`); a resume moves to `Connecting` and `Resuming`
  (`gateway/shard.rs:706`, `:734`), so it may never buffer output.
  Plausible; the resume test settles it.
- serenity 0.12.5 resumes on the first URL, against Discord's advice
  (`developers/events/gateway.mdx:261`); the fix is on `next` only.
- T-015's route C is now possible; amend it with the mock's entry.

## Provenance

| reference | commit | committed | licence | read to | tracker fetched |
| --- | --- | --- | --- | --- | --- |
| `discord/discord-api-spec` | `bf93257f85d6` | 2026-10-20 | MIT | README, the 15 operations, 6 schemas, headers; probe | 56 items, 133 comments |
| `discord/discord-api-docs` | `6128d99116af` | 2026-10-23 | see repo | gateway, gateway events, threads, rate limits, error codes, interactions, CDN | not fetched |
| `spacebarchat/server` | `190f6310519b` | 2026-10-19 | AGPL-3.0 | passes 1 to 3 on the gateway, interactions, threads, rate limits, database; issues by keyword, bodies and comments of #940, #1103, #1151, #1507, #1516, #1517, #1550, #1557, #1572 | 1,602 items, 2,477 comments, 846 review comments |
| `tomacheese/fauxcord` | `210824790681` | 2026-10-23 | none found | passes 1 to 4: README, contract test, skip list, drift workflow, gateway server, rate limit, test API, serenity harness | 384 items, 56 comments, 197 review comments |
| `szymboot/fauxcord` | `616321ff41a9` | 2026-10-24 | none found | a fork of the above; listing only | not fetched |
| `SilentHacks/simcord` | `69bb92dcb458` | 2026-10-22 | MIT | architecture, router, snowflakes, field-honesty tests | 40 items, 12 comments |
| `CraftSpider/dpytest` | `e443a7c3447c` | 2026-01-13 | MIT | tracker by keyword | 136 items, 284 comments |
| `twilight-rs/http-proxy` | `e3e7114d0796` | 2026-05-19 | ISC | README, layout | 103 items, 115 comments |
| `glideapps/fake-discord` | `2ca21262851e` | 2026-03-02 | none found | README; agent files deleted unread | 11 items |
| `Gelbpunkt/mock-gateway` | `973cc11983b3` | 2023-02-14 | AGPL-3.0 | README, resume handling | not fetched |
| `leonlarsson/discord-mocker` | `953f60f543a8` | 2026-08-23 | none found | README | not fetched |
| `suneettipirneni/discord-mock-server`, `pocketcord/pocketcord`, `minicord-dev/minicord`, `mavolin/dismock`, `tophcodes/blurplejs` | `7f55d631baea`, `67e84c256267`, `a4eb8daecb63`, `42b2aa755944`, `fb1e4affb915` | 2022-03-18, 2022-10-16, 2022-02-21, 2024-07-06, 2022-10-10 | MIT, ISC, MIT, MIT, see repo | listing, licence, size, README | not fetched |
| `serenity-rs/serenity` 0.12.5 | the crates.io release | | ISC | `src/http`, `src/gateway`, `src/client`, `src/model` as cited | search only: #1247, #1739, #2322, #2479, #2709, #2832, #3575 |

Not reached: `b1naryth1ef/mockcord` (named by search, 2017, not cloned); the
`@robojs/mock` server (named by the web search, repository not found); every
tracker's discussions. Agent instruction files were deleted from the fauxcord
clones and from `glideapps/fake-discord` before reading.

**The corpus** is not kept in this tree, by the operator's ruling of
2026-10-24. Re-fetch a reference and its tracker with:

```sh
git clone https://github.com/OWNER/REPO REPO
git -C REPO checkout COMMIT
sh scripts/fetch-tracker.sh OWNER/REPO trackers/OWNER__REPO
```

The crate counts came from a scratch crate per candidate resolved with
`cargo generate-lockfile` (no build), compared by crate name with this
repository's `Cargo.lock` (280 packages).
