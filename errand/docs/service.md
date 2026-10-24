# As a service

errand-box runs the daemon. `errand-box setup` makes the account it runs as,
writes the configuration to that account's `.config/errand/config.json` with
mode 0600, and installs a service for `errand-box serve` under systemd or s6.
`errand-box up` starts it on a host with no init. The commands are in
[operations](https://github.com/Azathothas/errand-box/blob/main/docs/operations.md).

The daemon runs as an ordinary account, not as root: the configuration file
holds the bot token, and every agent is started as that same account.

## Restarting

errand-box's supervisor restarts the daemon after every exit, a refusal
included, two seconds later. Exit 2, 3 and 4 are decisions the daemon made,
and a restart repeats one whose cause has not changed. Not every cause stays:
a sandbox that does not answer yet, while Incus starts, is also exit 2, and
it passes on its own. See [getting started](start.md) for what each code
means.

## Seeing more in the log

The log holds progress, warnings, and errors by default. Each `-v` on the
command line adds a level:

- `errand-box serve -v` logs each decision and why: which messages were accepted,
  where a session starts and on which model, each prompt sent, each provider
  answer with its status and time, and each refusal.
- `errand-box serve -vv` also logs each step: every command sent to the agent and
  every event it answers with, by type.
- `errand-box serve -vvv` also logs every line exchanged with the agent, verbatim.
  That includes prompts and tool output, so keep it to diagnosing a problem.

errand-box writes the service without a flag, and rewrites it on every setup,
so a flag is for a run by hand as the daemon's account.

## Limits

Under errand-box each session is its own Incus guest, with the memory, CPU,
process and disk limits of the `[box]` section of errand-box's configuration.
Each limit bounds one session, not all of them together.
