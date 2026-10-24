# Getting started

## What you need

- A chat bot token, and one channel for it to serve.
- A credential for a model provider.
- Linux, with either [bailey](https://github.com/QaidVoid/bailey) or rootless
  podman. The sandbox is not optional, so one of them has to be there. Under
  errand-box, `bailey` is errand-box itself.

## Install it

errand is built into errand-box; there is no program of its own to build.
`errand-box setup` installs errand-box, makes the account the daemon runs as,
writes the configuration below from errand-box's own input files, and
supervises the daemon: [operations](https://github.com/Azathothas/errand-box/blob/main/docs/operations.md).

## Configure it

Write `~/.config/errand/config.json`. The smallest file that starts:

```json
{
  "chat": {
    "token": "the bot token",
    "channelId": "the one channel to serve",
    "allowedUserIds": ["the accounts that may drive sessions"]
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

Everything else has a default, listed in [configuration](reference/configuration.md).
Keep the file mode 0600: it holds two credentials.

## Run it

```sh
errand-box serve   # errand-box's supervisor runs this; errand is not a program of its own
```

It checks the sandbox before it touches the chat service, and says what it can
and cannot enforce on this host. If it cannot enforce something the
configuration asks for, it stops rather than starting anyway. See
[sandboxing](sandboxing.md) for what that report means.

Then post in the served channel. The first message opens a thread and starts a
session; replies in that thread are prompts.

## When it will not start

The exit code says which kind of problem it was, so a service can tell a
refusal from a crash.

| code | what happened                                                    |
| ---- | ---------------------------------------------------------------- |
| 2    | the configuration, the sandbox backend, or the token was refused |
| 3    | the backend cannot enforce a guarantee the configuration demands |
| 4    | another daemon already holds this state directory                |

Two daemons on one bot token both act on every message, which is what 4
prevents. A lock left by a process that is gone is taken over.
