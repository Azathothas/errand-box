# errand's tracker, read for the vendor-or-rewrite decision

**Date:** 2026-10-24. **Question:** does errand's tracker make vendoring and
patching errand, or rewriting the Discord layer, the better route for
errand-box, and what does each route inherit?

**Commits:** errand `715e9b0b777a` (main, 2026-10-23), its release v0.3.1
`97cea7de0265` (the pinned one), and bailey `98d9b455adc6`. An errand path
below is at `715e9b0b777a` unless it starts with `v0.3.1:`. An errand-box
path starts with `eb:`. Issue numbers are bare references, as `errand#41`.

This page is one half of the decision; another unit measures errand's code.
With two minutes, read the verdict and the table in section 3. With a reason
to distrust this page, read the claims table and run "Reproduce".

## What this did not establish

- No errand code was built or run. The one run is the git reproduction of
  errand#52 (section 3). Every other code claim is a reading at a cited line.
- Discussions were not fetched (GraphQL only). kage's tracker was not read,
  although errand#35 and errand#45 depend on kage.
- errand#5 (80 KB) was read by its headings, sections 0, 8 and 13, and its two
  comments. The six review comments on errand#13 were not read one by one.
  Of bailey's 98 comments, only the openings of the owner's 35 and those on
  bailey#6 and #32 were read.
- The corpus stops at 2026-10-24T08:22Z. An open issue may be fixed since.
- Code size and the cost of pruning belong to the other unit.
- Whether an account is a person or a bot was not established.
- This is the first write-up of this tracker, so no earlier revision is
  corrected here. Assume that more claims are wrong than the ones listed next.

## The claims this page is least sure of

1. errand#48 is critical for errand-box only when the provider's upstream is a
   bare origin. That an `anthropic` kind uses one is inferred from errand's
   fixture (`src/config/validate/tests.rs:782`). Settle it in kage v0.5.0.
2. errand#54 item 2 says reqwest 0.13.5 keeps `x-api-key` on a cross-host
   redirect. reqwest was not read; the client's lack of a redirect policy is
   confirmed (`src/sandbox/broker/server.rs:59-62`).
3. errand#36's size half assumes the notes harvest reads a sparse
   `/state/remember.md` before errand's disk accounting ends the session
   (`src/session/disk.rs:42` counts apparent size). The order was not traced.
4. errand#41's ruling says an incus-user project cannot grant the backend's
   exceptions. errand-box's incus-user project holds instances with nesting
   and `raw.idmap` (eb:`src/setup.rs:1167-1172`), and no errand-box source sets
   `restricted.containers.lowlevel`. Run `incus project show` on a live host.
5. The verdict depends on which features errand-box keeps. That is the
   operator's decision, and the tracker cannot make it.

## The candidate conclusions, written before testing

| | conclusion | the test the tracker can answer |
| --- | --- | --- |
| C1 | Vendor errand whole, patch it, and keep taking upstream releases | Does upstream's direction meet errand-box's, and what does one release cost to reconcile? |
| C2 | Vendor and prune: take one commit, delete what errand-box does not run, patch the inherited defects, and read upstream's security fixes only | Is the tracker's security debt in code errand-box keeps? Are the open defects local patches? |
| C3 | Rewrite a narrow pass-through and adopt errand's mechanisms by citation | Is the debt in code that a rewrite can leave out by design? |
| C4, the control | Stay on v0.3.1 behind the bailey stand-in | Is there an upstream release errand-box can move to without losing a property? |

## Verdict

**C2, vendor and prune.** C1 and C4 are refuted. C3 is the better route
under the condition at the end of this section.

1. **The security cost sits in three areas errand-box needs on any route:**
   the broker, the daemon's file access in directories a session writes, and
   host-side git. 9 of the 10 closed fixes in section 4 are there. errand-box
   keeps `/workspace` and `/state` on the host too, so each class applies.
2. **That class took several rounds, and review found them, not tests.**
   errand#2 closed on 2026-09-15; errand#6 and #7, bypasses of its fix, opened
   on 2026-09-16. errand#52 passes the check that closed errand#7. errand#54
   lists four mechanisms that survive errand#48's fix. A rewrite re-derives
   each one.
3. **The severe open defects that reach errand-box are local patches.**
   errand#48, #52, #36, #53 and #54 each have a fix in one file (section 3).
   The architectural ones, errand#42, #45 and #49, overlap T-020 and T-021.
4. **Upstream's direction leads to no release errand-box can take.** errand#41
   puts the errand account in `incus-admin`, with one instance per launch and
   errand's own image. The operator kept the restricted project
   ([incus-backend.md](../../incus-backend.md) section 10). So C4 has no exit,
   and C1 reconciles toward a backend errand-box would switch off.
5. **The chat layer carries little tracker debt** (errand#4, #11, #30, #40),
   so the tracker gives no reason to rewrite the whole program.

**What would make C3 the better route.** The operator may rule that
errand-box drops the broker's provider route and the host-side push. Then 7 of
the 11 defects that reach errand-box leave with that code, and so do the three
git fixes. A rewrite re-derives only the CONNECT gate (errand#9, #12, #17, #18)
and the file handling (errand#13's audit, errand#36). That ruling needs
another home for the provider key, which T-020's link does not give. C3 also
wins if the other unit finds the kept modules tangled with the deleted
backends.

## 1. Every issue and pull request

39 issues, 22 of them open, and 16 pull requests, 1 of them open. The owner
opened 13 of the pull requests and none of the issues. The account that
[incus-backend.md](../../incus-backend.md) names as the operator's opened
errand#11, #34, #35 and #41. Kinds: security, broker (broker and egress),
sandbox, chat (chat and Discord), git (host-side git and pull requests), config, usage (usage and queue),
release.

| # | state | kind | role | subject | outcome |
| --- | --- | --- | --- | --- | --- |
| 1 | closed | sandbox | other | The CA bundle is not granted on Arch hosts | fixed, 7a24e12; widened to openSUSE |
| 2 | closed | security, git | other | Host-side git runs the repository's hooks and credential helper | fixed, aabf107 |
| 3 | closed | git | other | Start sessions from GitHub mentions and assignments | built, c1f4b46, 24e6ade, e772f7e |
| 4 | closed | chat | other | Retry a failed chat gateway instead of exiting | fixed, d067597 |
| 5 | open | sandbox | other | "All Possible Futures": isolation and nesting options | no owner comment |
| 6 | closed | security, git | other | `log.showSignature` with `gpg.program` runs a program on the host | fixed, 5811ab2 |
| 7 | closed | security, git | other | A `.git` file sends host-side git to another repository | fixed, e2808ca |
| 8 | closed PR | security, git | other | The one-line fix for #6 | not merged; 5811ab2 landed |
| 9 | closed | security, broker | other | The broker judges the CONNECT name, not the address it dials | fixed in the Rust port, 9c8a91d |
| 10 | closed PR | git | other | A test pull request opened through errand | not merged |
| 11 | closed | chat | other | Accept messages from a third-party bot | reshaped: a bot only inside a thread, by invitation (8fc3095) |
| 12 | closed | broker | other | A name that does not resolve is refused as internal | fixed, 5d61aaf |
| 13 | merged PR | release | owner | Port the daemon to Rust | merged after two audits |
| 14, 15, 20, 21, 24, 25, 26, 29 | merged PR | release | owner | v0.2.0 to v0.3.1, one each | merged |
| 16 | closed | sandbox | other | A full private /tmp ends the session with a wrong message | fixed in 7 commits |
| 17 | closed | broker | other | CONNECT stalls: no timeout, one address | fixed, 6ec654a |
| 18 | closed | broker | other | Plain http through the broker always answers 404 | fixed, ffe4c72 |
| 19 | closed | git | other | A repository below the top of the project is not found | fixed, 8874abe and three more |
| 22 | closed | sandbox | other | /workspace is granted without execute | fixed, 7a0e1f1 |
| 23 | closed | security, sandbox | other | A bailey config planted in the session's HOME runs as the operator | fixed, 5f4ea77 |
| 27 | closed | usage | other | A spent window that credits still serve counts as spent | fixed by #28 |
| 28, 31 | merged PR | usage | owner | Bill past a spent window; read kage's token totals | merged |
| 30 | merged PR | chat | owner | End the session task once the session ends | merged |
| 32 | open PR | release | owner | v0.3.2 | open since 2026-10-19 |
| 33 | open | sandbox | other | Collect a cold session's state and project by age | no comment |
| 34 | closed | chat | other | Post images an agent makes | built through #35 |
| 35 | merged PR | chat | other | Generate images and post them | merged after rework for kage#224 |
| 36 | open | security | other | `!file` hangs on a FIFO and reads a file that grew past the limit | pipe half fixed on main (4a8011c); size half open |
| 37 | open | config | other | A provider named `gemini` or `google` stops every session | open |
| 38 | open | security, usage | other | `generate_image` has no cap | open |
| 39 | open | config | other | An images config with no endpoint passes validation | open |
| 40 | open | chat | other | An image caption can mention a user | open |
| 41 | open | sandbox | other | An Incus backend | accepted with changes, 2026-10-23 |
| 42 | open | security | other | The session holds the GitHub token verbatim | open |
| 43 | open | broker | other | No ask, allow-once or time limit on a credential | open |
| 44 | open | broker | other | A provider route allows any method and path | open |
| 45 | open | broker | other | The nonce is shared and looks like a key | open |
| 46 | open | broker | other | A response is relayed untouched, an echoed key included | open |
| 47 | open | broker | other | No record of what a credential was used for | open |
| 48 | open | security, broker | other | No path boundary on a provider route | open |
| 49 | open | broker | other | One broker and one nonce per provider for all sessions | open |
| 50 | open | security, sandbox | other | podman: real keys on the `podman run` argv, and no gap | open |
| 51 | open | broker | other | Default egress is `*` on 443; 168.63.129.16 is not refused | open |
| 52 | open | security, git | other | `clone --shared` follows a planted `alternates` file | open |
| 53 | open | security | other | Redaction skips upload bytes and secrets under 8 bytes | open |
| 54 | open | security, broker | other | Follow-ups to #48 | open |
| 55 | open | sandbox | other | No gate check proves the isolation claims | open |

## 2. The maintainer's rulings

The owner wrote 10 of the 41 comments. Each ruling below is read from them.

| issue | the ruling | the stated reason |
| --- | --- | --- |
| errand#1 | accepted and widened to openSUSE | a probe of four images showed openSUSE fails the same way |
| errand#9 | accepted: resolve once, judge every address, dial a judged address; `allowInternal` admits a literal only | what a name points at is not the operator's to decide |
| errand#12 | accepted: an unresolved name answers 502, an internal one 403; startup warns | the same reason, for refusing `allowInternal` on a name |
| errand#16 | accepted in seven commits; an automatic restart on ENOSPC refused | it saves one message at the cost of a second start path |
| errand#17 | reshaped: addresses tried in turn, not raced; no whole-request timeout on the provider client | a race is not needed for one slow address; a timeout cuts a long streamed reply |
| errand#18 | accepted: plain http goes through the same gate, and `egressPorts` is honoured | none beyond the fix |
| errand#19 | accepted; a `.git` file stays refused | errand pushes from the host, so a `gitdir:` pointer lets a session push a repository outside its tree |
| errand#3 | built; the owner's one comment asks whether it is done | none stated |
| errand#35 | merged after the pull request was reworked | kage#224 changed the tool's output to an ACP image block |
| errand#41 | accepted with eight decisions, below | per decision |

### errand#41, in full

The issue asks eight questions. Its first comment is a triage note whose
author calls itself a bot account; its third is an error string. The owner's
comment of 2026-10-23T15:02Z decides all eight.

| | the decision | what it means for errand-box |
| --- | --- | --- |
| 1 | a third concrete backend; the enum stays closed; no plugin system | no seam for an external backend, so the backend lives in errand's source |
| 2 | the project allows nesting, disks bounded by paths, and `raw.idmap` bounded by uid and gid; proxy devices stay blocked | errand-box's project has the same, and also allows proxy devices until T-020 |
| 3 | the broker reaches the guest over a unix socket on a disk device, with a relay in the guest | T-020's design; the broker needs a unix listener or a host relay |
| 4 | one instance per launch; state outside the host mounts does not survive | errand-box keeps one per session, and a revived session starts in about 0.3 s; this backend would not |
| 5 | `probe` needs the `incus` client as root or in `incus-admin`; an incus-user project is unavailable | the errand account becomes equal to root, which errand-box's project exists to avoid |
| 6 | errand builds the guest image in its own repository; the backend takes an alias | not errand-box's image or launcher |
| 7 | a separate key, `sandbox.incus.image` | none |
| 8 | `raw.idmap both`, bounded; no `shift=true` | the same as errand-box |

Two notes follow: `sandbox.cpus` maps to `limits.cpu.allowance`, and instance
names are cut to 63 characters with a trailing dash removed.

**A finding against decision 5.** Its reason is that "per-user `incus-user`
projects cannot grant the exceptions above". errand-box runs in an incus-user
project with nesting, disks under the errand home, and `raw.idmap` for the
errand uid. Its live run of 2026-10-23 passed. Root added one exception, proxy
devices. This is least-sure claim 4.

## 3. The open issues errand-box inherits if it vendors

Severity is for errand-box's deployment: the bailey path, guests with no
network device, and the config eb:`src/errandcfg.rs` writes. Critical: a guest
gets the provider key off the host. High: a guest publishes another session's
data. Medium: a guest stops every session, or a credential reaches it by
design. Low: it needs an operator setting, or gives the guest nothing new.

| issue | the claim, checked at | verdict | reaches errand-box | severity | the fix |
| --- | --- | --- | --- | --- | --- |
| errand#48, with #54 item 1 | `route_for` matches with `starts_with` (`src/sandbox/broker/mod.rs:507`); the target is `format!("{trimmed_upstream}{rest}{query}")` (`:547`); `build_forwarded` adds the real key (`:657-692`) | confirmed | yes: kage's base URL is the broker's `/provider/NAME`, reached from the guest through the proxy device (`src/sandbox/broker/server.rs:175-181`) | critical when the upstream is a bare origin; none when it has a path | local: a segment boundary and a parsed target |
| errand#52 | `git clone --shared --bare` (`src/session/pr.rs:636-651`); `assert_contained` reads only `--absolute-git-dir` (`:246-267`) | confirmed, and reproduced with git 2.43.0 | yes, with GitHub configured: the guest writes a host path into `objects/info/alternates`, and the errand account can read every session's project | high | local: refuse `alternates`, `commondir` and links under `.git` |
| errand#36, size half | the size check is at `src/session/session/files.rs:230`, `read_to_end` at `:244`; `read_beneath` has no cap (`src/sandbox/paths.rs:229-234`) | confirmed | plausible: a sparse file in `/state` makes the daemon read gigabytes | medium | local: read through a limit |
| errand#36, pipe half | `v0.3.1:src/sandbox/paths.rs:187` opens with `O_RDONLY` only | confirmed at v0.3.1; fixed at `715e9b0b777a` | yes at the pinned v0.3.1 | low: one session's task waits | taken by moving past 4a8011c |
| errand#42 | `env.insert(TOKEN_VARIABLE..., github.token.clone())` (`src/session/session.rs:2396`); the schema says the token is reachable by the agent (`src/config/schema.rs:373`) | confirmed | yes; [SECURITY.md](../../../SECURITY.md) already states it and asks for a fine-grained token | medium | architectural: a per-session App token, or T-021's ruling on errand's defaults |
| errand#53 | `Upload` keeps `bytes` unchanged (`src/session/redacted.rs:105-112`); the floor is 8 bytes (`src/config/redact.rs:17`) | confirmed | yes: `!file` of a file that holds `GH_TOKEN` posts it to the thread | low to medium | local: scan the bytes before the upload |
| errand#46 | `stream_answer` copies status, headers and body (`src/sandbox/broker/mod.rs:694-717`) | confirmed | yes, if the upstream echoes a request header | low to medium | local, a streaming scrubber in one function |
| errand#54 item 2 | the reqwest client is built with `connect_timeout` only (`src/sandbox/broker/server.rs:59-62`) | the setting is confirmed; reqwest's header handling is plausible | yes for an `anthropic` kind, which sends `x-api-key` | low alone; worse with #48 | local: no redirects on that client |
| errand#49 | one `Broker` for the daemon (`src/serve.rs:654-660`, `:689-698`); one nonce per provider (`:419-436`) | confirmed | yes, without escalation: every guest holds the same nonce already | low | architectural: a broker per session, which T-020's per-session socket fits |
| errand#45 | 32 random bytes as hex (`src/serve.rs:487-499`); `secret_values` reads only the config (`src/config/redact.rs:54`) | confirmed | yes: the nonce is not scrubbed from chat | low | local for the prefix; per session depends on #49 |
| errand#44 | the session's method and headers are copied (`src/sandbox/broker/mod.rs:657-692`) | confirmed | yes | low | architectural: method, path and placement on a route |
| errand#51 | `EGRESS_ALLOW = ["*"]` (`src/config/schema.rs:628`); `is_private_v4` has no arm for 168.63.129.16 (`src/sandbox/broker/mod.rs:243-268`) | confirmed | no for `*`: eb:`src/errandcfg.rs:56-74` writes a list; the address only through an allowed name, on 443 | low | local: one arm |
| errand#43, #47 | requests for an ask step and for receipts; `debug` logging only (`src/sandbox/broker/mod.rs:552-566`) | confirmed as described | features, not defects here | none | architectural |
| errand#37 | `KAGE_KNOWN` lists `gemini` | not checked | only if the operator names the provider so | low | local |
| errand#33 | nothing deletes a session's state or project | not checked | yes: errand-box's `maint` deletes instances, not errand's directories | low | local |
| errand#38, #39, #40 | images | not checked | no at v0.3.1, which has no images code; yes if a vendored copy keeps images | - | local |
| errand#50, #55 | podman has no broker; a test asserts no gap (`src/sandbox/podman.rs:263`) | confirmed | no: errand-box does not run podman | none | delete the backend |

**The 11 defects that reach errand-box** are errand#36, #42, #44, #45, #46,
#48, #49, #51, #52, #53 and #54. errand#51 reaches only through an allowed
name. Six of the eleven are in the provider route of the broker: errand#44,
#45, #46, #48, #49 and #54. One, errand#52, is in the host-side push.

**The run behind errand#52** ("Reproduce"): a planted `alternates` line, and
then a `commondir` file, each let `clone --shared --bare` read another
repository, while `--absolute-git-dir` still named the project's own `.git`.

### Where an issue and the code disagree

1. errand#54 item 1 says the judged host "is still the whole string". `url_host`
   splits on the last colon (`src/sandbox/broker/mod.rs:720-732`), so
   the authority `api.anthropic.com:443`, then `@`, then `evil.example`, judges
   as `api.anthropic.com` and the client dials `evil.example`. No refusal happens at all.
2. A comment on errand#5 says a path of `/provider/anthropic`, then `@`, then
   `attacker.example/` exfiltrates; errand#48's body says the resolver refuses it. The code agrees
   with errand#48: that host string does not resolve.
3. errand#36 is open, but its pipe half is fixed at `715e9b0b777a`
   (`src/sandbox/paths.rs:191-196`, from 4a8011c), not at v0.3.1.
4. `assert_contained`'s doc comment (`src/session/pr.rs:239-245`) says an
   alternate out of the tree is refused. The check reads `--absolute-git-dir`
   only, which reports no alternate (errand#52).
5. `src/sandbox/broker/server.rs:1-4` says one `Broker` serves one session;
   `src/serve.rs:654-660` starts one for the daemon (errand#49).
6. errand#52's comment says its control fails at the clone (git 2.55.0). With
   git 2.43.0 the control clone exits 0 and the read exits 128.
7. The brief for this unit says T-006 exists because errand refused a key.
   `output.postImages` arrived on main in 4a8011c (2026-10-21), after v0.3.1
   (2026-10-18), and errand refuses unknown keys since 8ab4a87 (2026-09-03).
   T-006 is drift between main and the pinned release.

## 4. The closed fixes, and where each mechanism lives now

A rewrite re-derives these. Each applies to errand-box, whose sessions write
the same host directories and reach the same broker.

| item | the defect | the mechanism at `715e9b0b777a` |
| --- | --- | --- |
| errand#2 | host git ran a hook and a credential helper from the session's repository | `-c` overrides in `SAFE_CONFIG` (`src/session/pr.rs:184-206`); no system or global config (`:214-218`); the push runs from a fresh bare clone (`:621-671`) |
| errand#6, #8 | `git log` ran `gpg.program` through `log.showSignature` | `log.showSignature=false` and `merge.verifySignatures=false` in `SAFE_CONFIG`; the body is read in the clone (`:678`) |
| errand#7 | a `.git` file or link pointed host git at another repository | `assert_contained` (`:246-267`); errand#19 keeps a `.git` file refused |
| errand#9 | the broker judged a name and dialled whatever it resolved to | resolve once, judge every address (`src/sandbox/broker/mod.rs:243-268`, `:345`, `:428`), dial a judged address (`server.rs:255`); `allowInternal` for a literal only |
| errand#12 | an unresolved name was reported as internal | `NoAddress` tells the two apart (`mod.rs:464`); a startup check names the resolver (`src/serve.rs:458`) |
| errand#17 | one slow lookup or address stalled a CONNECT | 5 s lookup, 5 s per address, 10 s budget (`mod.rs:397-409`); each address tried in turn |
| errand#18 | plain http always answered 404 | absolute-URI forwarding through the same gate (`mod.rs:110`) |
| errand#23 | bailey read a planted global config from the session's HOME | bailey's XDG directories point at the operator's (`src/sandbox/bailey.rs:92-124`); errand-box's version is rule 1 of [SECURITY.md](../../../SECURITY.md) |
| errand#13, the audit | daemon reads followed links the session planted; credentials reached the startup log | `openat2` with `RESOLVE_BENEATH` and `RESOLVE_NO_SYMLINKS` (`src/sandbox/paths.rs:118-140`), `write_beneath` (`:246`); provider credentials redacted (`src/config/redact.rs:23-45`) |
| errand#36, pipe half | a FIFO blocked a daemon read | `O_NONBLOCK` on every read beneath a root (`src/sandbox/paths.rs:191-196`) |

Two choices stay open. The web interface has no login by design
(`src/web/address.rs:1-9`), and errand-box enables it on loopback
(eb:`src/errandcfg.rs:152`). errand#2's report shows a session writing to the
agent's stdout through `/proc`, so a chat layer treats the ACP stream as
guest-controlled.

## 5. Release cadence and breaking changes

Measured from `api/releases.json` and from the tags in the tree.

| measure | value |
| --- | --- |
| releases | 9, from v0.1.0 (2026-09-18) to v0.3.1 (2026-10-18), 30.5 days |
| gaps between releases, in days | 0.3, 2.6, 3.6, 4.4, 0.5, 0.9, 9.9, 8.2; median 3.1 |
| the next release | v0.3.2, an open pull request since 2026-10-19 |
| since the pin, v0.3.1 to `715e9b0b777a` | 7 commits; in `src`, 37 files, 4137 lines added, 248 removed |
| config fields added and removed, per release, as `pub NAME:` lines in `src/config/schema.rs` | v0.2.1 +4; v0.2.2 +6; v0.2.3 to v0.2.5 none; v0.3.0 +5 -1; v0.3.1 +1; main since +2 |
| the changelog's breaking marker | once, v0.2.0, providers in one place |
| a break without the marker | v0.3.0 replaced pi with kage and refuses `agent.extensions` (`v0.3.0:src/config/validate.rs:763-765`) |
| the bailey policy writer, `src/sandbox/policy.rs` | changed in 4 of the 6 intervals from v0.2.2 to `715e9b0b777a` |
| of those four | v0.2.5 changed the shape (`[[device]]` tables); v0.2.3 added execute on `/workspace`; v0.2.4 and v0.3.0 changed comments and the resolver file |
| `src/sandbox/bailey.rs` | changed in v0.2.4 and v0.3.0; neither file changed after v0.3.0 |

**What this costs each route.** errand refuses an unknown config key, so a key
on main breaks a config written for the release (T-006). Staying pinned moves
the generator, the stand-in's parser and the pins together at each release. A
vendored copy instead reads each upstream security fix and ports it by hand. A
rewrite has neither cost and none of the fixes.

## 6. Direction, and what errand-box keeps or deletes

errand's README says what it is for: a coding agent run from a chat channel,
in a sandbox it cannot escape, with no unsandboxed mode. The ruling on
errand#41 adds a closed backend enum and an image errand builds itself.
errand#5 proposes gVisor, Kata and microVM futures; the owner has not
commented on it.

| feature | errand-box today | in a vendored copy |
| --- | --- | --- |
| the bailey backend | used, through the stand-in | delete; an Incus backend replaces it |
| the podman backend | not used | delete, with errand#50 and errand#55 |
| the broker, CONNECT and the provider route | used | keep; patch errand#48, #54, #46; a broker per session with T-020 |
| the host-side pull request push | used when GitHub is configured | keep and patch errand#52, or move the push into the guest (T-021) |
| GitHub-started sessions (errand#3) | not configured ([SECURITY.md](../../../SECURITY.md)) | delete; it needs a classic token (errand#42) |
| images (errand#34, #35) | removed for v0.3.1 (T-006); the operator's account asked for them | keep if wanted, with errand#38 to #40 |
| usage windows (errand#27) | not configured | optional |
| the web interface | on loopback, eb:`src/errandcfg.rs:152` | keep on loopback, or delete |
| memory, recall, delegation, skills | errand's defaults | keep, with the chat layer |
| the queue | `maxConcurrentTurns` 2, `maxLiveSessions` 4, eb:`src/errandcfg.rs:153` | keep |

## 7. bailey

bailey's tracker has 25 issues, 3 open, and 41 pull requests. Its last item
opened on 2026-10-11 and its last comment is of 2026-10-12. Its last release,
v0.2.4 on 2026-10-19, is `98d9b455adc6`, the commit
[bailey-contract.md](../../bailey-contract.md) pins. So no bailey item changes
the contract errand-box implements. The items on that surface:

- bailey#69 changes what real bailey's `doctor` prints for `cgroup
  delegation`. errand reads only `no` there; errand-box prints a fixed line.
- bailey#6 and bailey#32 are bailey releases that broke errand's `/workspace`
  and its daemon calls. errand-box reads errand's policy, not bailey's code, so
  it is outside that coupling.
- bailey#25 and bailey#48 keep the trust store and a relative XDG value out of
  a sandbox. errand#23 is the same class on errand's side.
- bailey#12, #19 and #22 stay open. They are bailey's own enforcement.

**Dropping the impersonation is possible only with C2 or C3.** It removes
eb:`src/policy.rs` and eb:`src/trust.rs` (569 and 171 lines by `wc -l`), the
`doctor` line that names a Landlock ABI, the digest check on a policy file the
guest can write, and the text of T-016. T-019's limits then need no policy
key. The policy writer is the upstream surface the stand-in tracks, and
section 5 shows it moving.

## 8. What would break each route

**C2 breaks when an upstream fix lands in code the pruned copy restructured.**
Nothing notifies errand-box after vendoring, and the tracker shows a fix and
its bypass a day apart (errand#2, then #6 and #7).
[vendoring.md](../../methodology/vendoring.md) is the rule: each patch carries
its reproduction and the bare issue number. C2 also breaks when the copy keeps
an assumption of the bailey path: the `169.254.169.1` mapping, HOME in the
state directory (errand#23), and gaps worded for Landlock. From the first day,
the 11 defects of section 3 are errand-box's.

**C3 breaks when it keeps a feature and misses one shape of a recorded
class.** The shapes: git configuration that names a program (errand#2, #6); a
git directory or object store that points outside (errand#7, #52); a name
judged in place of an address (errand#9); an unresolved name read as internal
(errand#12); a lookup or dial with no deadline (errand#17); a FIFO or a
growing file (errand#36); a link a daemon read follows (errand#13); a URL
built by joining strings (errand#48, #54). C3 also breaks when its tests pass
on the defect: errand#55 records a test that asserts no gap where one exists,
and the owner's comment on bailey#44 records another. It breaks when it drops
a feature the operator then wants: the operator's account opened errand#34 and
#35 for images.

**C4 is already broken.** T-006 is its drift, v0.3.1 still has errand#36's
pipe half, and its only path to an Incus backend needs `incus-admin`.

## Claims and verdicts

| claim | verdict | evidence |
| --- | --- | --- |
| a guest can send the provider key to a host it names | confirmed in code for a bare-origin upstream; not run | `src/sandbox/broker/mod.rs:507`, `:547`, `:657-692`, `:720-732` |
| an `anthropic` kind uses a bare origin | plausible | errand's fixture `src/config/validate/tests.rs:782`; settle in kage |
| a planted `alternates` or `commondir` leaks another repository through the push | confirmed, run | "Reproduce", git 2.43.0 |
| errand-box writes its own egress list, not `*` | confirmed | eb:`src/errandcfg.rs:56-74`, `:127-130` |
| v0.3.1 lacks the FIFO fix | confirmed | `v0.3.1:src/sandbox/paths.rs:187` |
| a sparse file in `/state` exhausts the daemon's memory | plausible | `src/sandbox/paths.rs:229-234`; the order against `src/session/disk.rs:42` is not traced |
| reqwest keeps `x-api-key` across hosts | plausible | the setting at `server.rs:59-62`; reqwest not read |
| an incus-user project can carry the exceptions errand#41 needs | plausible | eb:`src/setup.rs:1167-1172`; not checked on a live project |
| the policy writer changed in 4 of 6 intervals | confirmed, measured | `git log v0.2.2..715e9b0b777a -- src/sandbox/policy.rs` |
| 9 of the 10 closed fixes sit in the broker, file access or host git | confirmed | section 4 |
| the chat layer has few tracker items | confirmed | section 1: errand#4, #11, #30, #40 |
| T-006 came from errand refusing a key | refuted as worded | the key arrived on main after v0.3.1 (4a8011c) |

## Reproduce

The corpus sits under `references/`, which `.git/info/exclude` keeps out of
git: the untracked shape [research.md](../../methodology/research.md) section
3.4 calls not working. On another machine, re-fetch; the tracker may differ.

```sh
sh scripts/fetch-tracker.sh QaidVoid/errand references/QaidVoid__errand
sh scripts/fetch-tracker.sh QaidVoid/bailey references/QaidVoid__bailey
git clone https://github.com/QaidVoid/errand references/QaidVoid__errand/tree
git -C references/QaidVoid__errand/tree checkout 715e9b0b777a
E=references/QaidVoid__errand
jq -r 'sort_by(.number)[] | "\(.number)\t\(.state)\t\(if .pull_request then "PR" else "IS" end)\t\(.author_association)\t\(.title)"' "$E/api/issues.json"
jq -r '.[] | select(.author_association=="OWNER") | "#\(.issue_url|split("/")|last) \(.created_at)\n\(.body)\n"' "$E/api/comments.json"
jq -r 'sort_by(.published_at)[] | "\(.tag_name) \(.published_at)"' "$E/api/releases.json"
git -C "$E/tree" diff --shortstat v0.3.1 715e9b0b777a -- src
git -C "$E/tree" log --format=%h v0.2.2..715e9b0b777a -- src/sandbox/policy.rs
git -C "$E/tree" show v0.3.1:src/sandbox/paths.rs | grep -n O_RDONLY
```

The errand#52 run, in an empty scratch directory. Both `show` commands print
`OTHER_SECRET`, and `rev-parse` names the project's own `.git`.

```sh
D=$(mktemp -d) && cd "$D" && export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
git init -q other && echo OTHER_SECRET > other/f && git -C other add f
git -C other -c user.email=a@b -c user.name=a commit -q -m s
git init -q proj && git -C proj -c user.email=a@b -c user.name=a commit -q --allow-empty -m mine
echo "$D/other/.git/objects" > proj/.git/objects/info/alternates
git -C proj update-ref refs/heads/leak "$(git -C other rev-parse HEAD)"
git -C proj rev-parse --absolute-git-dir
git clone -q --shared --bare proj out.git && git --git-dir=out.git show leak:f
git init -q proj2 && git -C proj2 -c user.email=a@b -c user.name=a commit -q --allow-empty -m mine
echo "$D/other/.git" > proj2/.git/commondir
git clone -q --shared --bare proj2 out2.git && git --git-dir=out2.git show HEAD:f
```

## Provenance

| source | commit or date | how far it was read |
| --- | --- | --- |
| errand tree | `715e9b0b777a`, 2026-10-23; full history, tags v0.2.2 to v0.3.1 | each cited file opened at its lines; nothing built |
| errand v0.3.1 | `97cea7de0265` | the inherited defects checked with `git show` |
| errand issues and pull requests | 55 items, fetched 2026-10-24 | every title; every body of a security, broker or git item; errand#5 by headings |
| errand comments | 41 | the owner's 10 in full; all on errand#3, #5, #13, #23, #41, #42, #50, #52, #53, #55 |
| errand review comments | 6, all on errand#13 | located, not read one by one |
| errand releases and tags | 9 and 9 | dates measured |
| bailey issues and pull requests | 66 items | every title |
| bailey comments | 98 | the first 700 characters of each of the owner's 35, and those on bailey#6 and bailey#32 |
| bailey tree | `98d9b455adc6`, 2026-10-19 | not opened; read through [bailey-contract.md](../../bailey-contract.md) |
| discussions, both repositories | - | not fetched: GraphQL only |
| errand-box | the working tree of 2026-10-24 | eb:`src/errandcfg.rs`, eb:`src/setup.rs`, eb:`src/policy.rs`, and the pages named above |
| git, for the errand#52 run | 2.43.0 | two variants and one control, in a scratch directory |
