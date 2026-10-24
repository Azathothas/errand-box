# errand

errand-box's hard fork of errand, taken from
[QaidVoid/errand](https://github.com/QaidVoid/errand) at `715e9b0b777a` on
2026-10-24. It is errand-box's code from that commit on, and it takes no
upstream release. What changed since the fork is in the git history of this
directory. [docs/index.md](docs/index.md) says what errand does.

- One channel, one thread per session, several sessions at once.
- The agent runs sandboxed. There is no unsandboxed mode.
- A queue bounds how much work reaches the model provider at once, so a burst
  of messages cannot get an account rate limited.
- The chat token never enters a sandbox.

## Documentation

| page | what it covers |
| --- | --- |
| [docs/index.md](docs/index.md) | what errand is |
| [docs/start.md](docs/start.md) | the first run |
| [docs/threads.md](docs/threads.md) | sessions, threads and their commands |
| [docs/sandboxing.md](docs/sandboxing.md) | the sandbox backends and the egress broker |
| [docs/service.md](docs/service.md) | running the daemon as a service |
| [docs/github.md](docs/github.md) | pull requests from a session |
| [docs/models.md](docs/models.md) | providers and models |
| [docs/images.md](docs/images.md) | images a session generates |
| [docs/memory.md](docs/memory.md) | what a session remembers |
| [docs/instructions.md](docs/instructions.md) | instructions given to the agent |
| [docs/reference/configuration.md](docs/reference/configuration.md) | every configuration field, generated |
| [docs/reference/commands.md](docs/reference/commands.md) | every chat command, generated |
| [docs/reference/characters.md](docs/reference/characters.md) | every reaction and prefix, generated |
| [CHANGELOG.md](CHANGELOG.md) | upstream's releases, up to the fork |

## What you need

- Linux, with either [bailey](https://github.com/QaidVoid/bailey) or rootless
  podman on the host. Without a backend the daemon refuses to start rather
  than running a session unconfined. Under errand-box, `bailey` is
  errand-box itself. `sandbox.backend` picks which, and defaults to `bailey`.
- A chat bot token, and one channel for it to serve.
- A credential for a model provider.

bailey confines a session as a host process, using Landlock for the filesystem
and seccomp for the syscall surface, so sessions use the host's own tools and
there is no image to build. podman runs the session in a container from an
image you provide instead. [Sandboxing](docs/sandboxing.md) covers what each
one enforces, and what the daemon does when a host cannot enforce it.

## Running it

```sh
errand-box serve     # the daemon, until it is told to stop; the supervisor runs it
errand-box threads   # list, inspect, and remove what past sessions left on disk
```

errand is a library inside errand-box, not a program of its own.

There is a file to copy in [config.example.json](config.example.json), and a
schema beside it that gives an editor completion and checking.

The configuration file is read from `~/.config/errand/config.json`, then
`/etc/errand/config.json`, then `config.json` in the working directory.
`ERRAND_CONFIG` names one outright and skips the search. A minimal one:

```json
{
  "chat": {
    "token": "the bot token",
    "channelId": "the one channel to serve",
    "allowedUserIds": ["accounts that may drive sessions"]
  },
  "agent": {
    "provider": "anthropic",
    "providers": {
      "anthropic": {
        "credentialName": "ANTHROPIC_API_KEY",
        "credential": "the provider key"
      }
    }
  },
  "projectRoot": "/srv/errand/projects",
  "stateDir": "/var/lib/errand"
}
```

Everything else has a documented default. The daemon refuses to start rather
than run with a guarantee it cannot keep: if the backend cannot enforce
everything configured on this host, it says which and stops, unless
`sandbox.requireFullEnforcement` is set to `false`.

## Two models, one session

A session can ask a cheaper model of the same provider one question about one
thing that already exists: a long log, a large file, a diff. It is shown that
one thing and nothing else, has no way to run or read anything, and answers in
text, so what comes back is a description to check rather than a decision to
follow. The point is what it keeps out of the session's own context, which is
paid for again on every later turn.

```json
{
  "agent": {
    "delegate": { "model": "glm-5.3-flash", "perTurn": 8, "deadlineMs": 60000 }
  }
}
```

Absent means no delegation at all. With it, the agent gets a `delegate` command
and is told how to use it; `!status` reports what it cost and how much it kept
out. To have the cheaper model do the work rather than describe it, use
`!model` instead.

## In a thread

`!help` lists what can be typed, `!usage` says how much of the provider's usage
window is left and when it resets, and `!model` moves the session to another
model of the same provider, keeping the conversation. Plan on the capable one,
switch to the cheap one to carry it out, switch back to review. The same commands are registered as slash
commands, so they can be picked rather than remembered. A message starting
`!!!` is an aside: the people in the thread see it and the agent is never told.

`!help`, `!usage` and `!model` also work in the channel, where they are
answered by the daemon rather than starting a session. `!model` lists the
models there; switching one still belongs to a session, so it asks for a
thread.

## Changing the configuration

The daemon reads its configuration file again when it changes, so most edits
take effect without a restart. The next session, and the daemon's own answers
to `!usage`, `!model` and the rest, use the new file.

A file that cannot be read is not fatal. An edit caught half-written, a field
left out, a path that does not resolve: each is refused whole, the log says
every reason, and the daemon carries on serving under the configuration it
already had. Only a configuration that cannot be read at startup stops it,
since a daemon with nothing to run on cannot serve.

What a reload does not reach, because it was committed to before the daemon
was serving:

- the connection and the chat token, so who may use the bot in the served
  channel is fixed until a restart
- the sandbox backend, the egress broker, and the provider routes it holds
- sessions already running, which keep what they were launched with

The model list is derived from the file again after a change, so an edited
`models` list is what `!model` lists straight away. `!models refresh` is for
asking the providers themselves again, over the network.

## Running it as a service

errand-box supervises errand under systemd or s6:
[../docs/operations.md](../docs/operations.md).

## Status

Early, but complete enough to run: chat and sandboxed sessions, with
errand-box as the sandbox and the supervisor. The local web interface was taken
out of the fork; a session is read and driven from its thread.

## Development

```sh
cargo fmt --check                    # formatting
cargo clippy -p errand --all-targets # lint, pedantic, warnings denied
cargo test -p errand                 # the test suite
cargo run -- serve                   # from the workspace root: errand-box runs the daemon
```

errand builds into the errand-box binary; the workspace root's
[`scripts/ci-local.sh`](../scripts/ci-local.sh) runs every gate step.

The reference pages are generated from the schema, the command table, and the
character table; `cargo test` fails when what is committed no longer matches,
and `cargo test regenerate_the_reference_pages -- --ignored` brings them back
in step.

The documentation site builds with VitePress from `docs`, with bun:

```
cd docs && bun install && bun run build
```

Project rules are errand-box's: [../AGENTS.md](../AGENTS.md).

## License

MIT OR Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
