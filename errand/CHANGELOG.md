# Changelog

What changed in each release, from the commit messages.
## [0.3.1] - 2026-10-18


### Features
- (config) Make !file upload limit configurable ([6ae99e1](https://github.com/QaidVoid/errand/commit/6ae99e186c36))

### Fixes
- (pr) Open pull requests against the forked-from repository ([86cd3fc](https://github.com/QaidVoid/errand/commit/86cd3fccd317))
- (session) End the session task once the session ends ([#30](https://github.com/QaidVoid/errand/pull/30)) ([c3da504](https://github.com/QaidVoid/errand/commit/c3da504b6b6c))
- (usage) Read kage's token totals and show tok/s ([#31](https://github.com/QaidVoid/errand/pull/31)) ([f47a0c3](https://github.com/QaidVoid/errand/commit/f47a0c31f99d))
- (usage) Let a mapped provider bill past a spent window ([#28](https://github.com/QaidVoid/errand/pull/28)) ([880c34d](https://github.com/QaidVoid/errand/commit/880c34d3709b))

## [0.3.0] - 2026-10-10


### Features
- (chat) Take discord stickers as attachments ([e442f96](https://github.com/QaidVoid/errand/commit/e442f9621948))
- (providers) Pass model sizes and broker anthropic keys ([fb97456](https://github.com/QaidVoid/errand/commit/fb97456c2ef9))
- (providers) Honor kind for custom kage providers ([651adc5](https://github.com/QaidVoid/errand/commit/651adc5288a6))
- Register kage plugins and leave them unbrokered by default ([92bb231](https://github.com/QaidVoid/errand/commit/92bb23127bc3))
- Run sessions on kage rpc instead of pi ([c4110ab](https://github.com/QaidVoid/errand/commit/c4110abf80c0))

### Fixes
- (agent) Accept a resume answer without a session id ([28f78e6](https://github.com/QaidVoid/errand/commit/28f78e60c44e))
- (pr) Open fork pull requests against the parent ([ff9abe3](https://github.com/QaidVoid/errand/commit/ff9abe3645c8))
- Run unconfigured tools without asking in sessions ([f16179a](https://github.com/QaidVoid/errand/commit/f16179a876fe))
- Render tool commands and whole spans of agent speech ([6914dc0](https://github.com/QaidVoid/errand/commit/6914dc0de00f))
- Widen the session resolver to three public resolvers ([02122a8](https://github.com/QaidVoid/errand/commit/02122a847b8b))

### Refactoring
- (agent) Drive sessions over kage ACP at pi parity ([7402932](https://github.com/QaidVoid/errand/commit/7402932e6189))

## [0.2.5] - 2026-09-30


### Fixes
- (sandbox) Fix shell tool spawn failing ([1168577](https://github.com/QaidVoid/errand/commit/116857731ddb))

## [0.2.4] - 2026-09-29


### Features
- Add errand skills ([b90c1c7](https://github.com/QaidVoid/errand/commit/b90c1c7a8d42))

### Fixes
- (sandbox) Bound the broker's dial across all addresses ([ace9ba7](https://github.com/QaidVoid/errand/commit/ace9ba71f513))
- (sandbox) Close the daemon's symlink-following sinks ([088ee6a](https://github.com/QaidVoid/errand/commit/088ee6a0c923))
- (sandbox) Point bailey user dirs at the operator home ([5f4ea77](https://github.com/QaidVoid/errand/commit/5f4ea7720bc6))
- (session) Refuse symlinks in recall and delegation answers ([6c4cafa](https://github.com/QaidVoid/errand/commit/6c4cafab783e))

## [0.2.3] - 2026-09-29


### Features
- (model) List models in the channel, and answer slash commands ([c436fd2](https://github.com/QaidVoid/errand/commit/c436fd2db197))
- (sandbox) Grant execute on /workspace by default ([7a0e1f1](https://github.com/QaidVoid/errand/commit/7a0e1f13494a))
- (usage) Let an operator re-read the usage window ([133fb67](https://github.com/QaidVoid/errand/commit/133fb6747763))

### Fixes
- (model) Let a short name added later reach a running session ([ce59a96](https://github.com/QaidVoid/errand/commit/ce59a9663a7b))
- (session) Resume on the configured model when a provider is gone ([8f117c6](https://github.com/QaidVoid/errand/commit/8f117c636022))

## [0.2.2] - 2026-09-24


### Features
- (chat) Drop the bars from the !usage table ([3051c91](https://github.com/QaidVoid/errand/commit/3051c9148a4b))
- (chat) Draw !usage as a bordered table with usage bars ([a46bf13](https://github.com/QaidVoid/errand/commit/a46bf13d7027))
- (chat) Show !usage as a table with reset times in UTC ([388f2ad](https://github.com/QaidVoid/errand/commit/388f2ad01408))
- (issues) Listen only in chosen repositories, for chosen triggers ([e772f7e](https://github.com/QaidVoid/errand/commit/e772f7eb55ac))
- (issues) Run GitHub sessions in a chat thread beside the issue ([24e6ade](https://github.com/QaidVoid/errand/commit/24e6ade6d99b))
- (issues) Start sessions from GitHub mentions and assignments ([c1f4b46](https://github.com/QaidVoid/errand/commit/c1f4b46325a5))
- (log) Add -v, -vv and -vvv for debug, trace and wire logs ([51642b7](https://github.com/QaidVoid/errand/commit/51642b7b10bb))
- (provider) Read several usage windows, a spent one first ([941ddac](https://github.com/QaidVoid/errand/commit/941ddac2b80a))
- (provider) Read usage by declared shape, or where a mapping says ([6a300f5](https://github.com/QaidVoid/errand/commit/6a300f5c49ba))
- (provider) Ask providers for their models, with per-model overrides ([01d0e64](https://github.com/QaidVoid/errand/commit/01d0e64f0b09))
- (sandbox) Forward plain http on the allowed egress ports ([ffe4c72](https://github.com/QaidVoid/errand/commit/ffe4c72d6646))
- (session) Start on a fallback model when a window is spent ([e623a16](https://github.com/QaidVoid/errand/commit/e623a16193c0))
- (session) Let !pr name the repository with --repo ([e21a540](https://github.com/QaidVoid/errand/commit/e21a5401ffef))
- (session) Tell the agent what came of its pull request ([c2e8a9e](https://github.com/QaidVoid/errand/commit/c2e8a9e99dfd))
- (session) Suggest diskTmp when a session runs out of scratch ([1f1b02b](https://github.com/QaidVoid/errand/commit/1f1b02b7d2f8))

### Fixes
- (agent) Forward the provider's error to chat and the log ([678b0ea](https://github.com/QaidVoid/errand/commit/678b0ea5c387))
- (chat) Write a GitHub login bare where it cannot be mentioned ([61a1b61](https://github.com/QaidVoid/errand/commit/61a1b61ed550))
- (issues) Post only what a turn came to on the issue ([1262828](https://github.com/QaidVoid/errand/commit/126282840a9e))
- (issues) Skip notifications with nothing new since the start ([da04669](https://github.com/QaidVoid/errand/commit/da046695b468))
- (sandbox) Tell a name that does not resolve from an internal one ([5d61aaf](https://github.com/QaidVoid/errand/commit/5d61aafbbe61))
- (sandbox) Give podman sessions the providers and extensions ([6956e65](https://github.com/QaidVoid/errand/commit/6956e6553e30))
- (sandbox) Keep every provider's real key out of a brokered session ([c9144c7](https://github.com/QaidVoid/errand/commit/c9144c7ae45b))
- (sandbox) Give every provider its key when egress is not brokered ([937468a](https://github.com/QaidVoid/errand/commit/937468ace316))
- (sandbox) Lay model entries over the agent's built-in definitions ([07b1269](https://github.com/QaidVoid/errand/commit/07b1269fa1c4))
- (sandbox) Honor tmpSize, shmSize and diskTmp under podman ([7979aad](https://github.com/QaidVoid/errand/commit/7979aadb080d))
- (sandbox) Start each launch on an empty disk-backed /tmp ([971192b](https://github.com/QaidVoid/errand/commit/971192b3e729))
- (sandbox) Bound broker lookups and dials ([6ec654a](https://github.com/QaidVoid/errand/commit/6ec654af1ca7))
- (serve) Meter every z.ai provider, not only the default ([88b8627](https://github.com/QaidVoid/errand/commit/88b86277b4c3))
- (serve) Broker built-in providers from the host's model store ([55177d0](https://github.com/QaidVoid/errand/commit/55177d050aca))
- (session) Name a repository by its directory, not its GitHub name ([9f9031e](https://github.com/QaidVoid/errand/commit/9f9031eaab3d))
- (session) Find a repository cloned below the top of the session ([8874abe](https://github.com/QaidVoid/errand/commit/8874abe36bd4))
- (session) Announce a model switch only once the agent takes it ([075576e](https://github.com/QaidVoid/errand/commit/075576edcf36))

## [0.2.1] - 2026-09-21


### Features
- (agent) Load pi extensions in the sandbox, and providers they register ([816bbeb](https://github.com/QaidVoid/errand/commit/816bbeb12872))
- (memory) Let the agent recall older facts on demand ([a69829f](https://github.com/QaidVoid/errand/commit/a69829fba204))
- (sandbox) Let a session's /tmp be backed by disk ([72dd7ab](https://github.com/QaidVoid/errand/commit/72dd7ab5d168))
- (sandbox) Make the private /tmp and /dev/shm sizes configurable ([011697b](https://github.com/QaidVoid/errand/commit/011697b84e93))
- (session) Say a resource-limited session can be continued ([e290a1c](https://github.com/QaidVoid/errand/commit/e290a1ca24ab))

### Fixes
- (memory) Tell the agent its recall is in the prompt, not the files ([979c221](https://github.com/QaidVoid/errand/commit/979c221eed38))
- (session) Start a bare model on the provider that serves it ([65c094b](https://github.com/QaidVoid/errand/commit/65c094b0d982))
- (session) Name the sandbox scratch limits on ENOSPC ([0a77725](https://github.com/QaidVoid/errand/commit/0a77725d39e2))
- (session) Retry a dropped attachment fetch and say why it failed ([10766eb](https://github.com/QaidVoid/errand/commit/10766eb90113))

## [0.2.0] - 2026-09-18


### Features
- (chat) Name the model on the opening line and every done ([29ca800](https://github.com/QaidVoid/errand/commit/29ca80033579))
- (chat) Group what a turn took and what it spent ([e5ffd5d](https://github.com/QaidVoid/errand/commit/e5ffd5d5f9a0))
- (config) [breaking] Describe every provider in one place ([d87057a](https://github.com/QaidVoid/errand/commit/d87057adea9f))
- (provider) Let a model say how hard it thinks ([cf8f505](https://github.com/QaidVoid/errand/commit/cf8f505bb340))
- (session) Make the model listing something to type back ([c0e31a9](https://github.com/QaidVoid/errand/commit/c0e31a918000))
- (session) Say how long a turn waited and how long it took ([6c6459e](https://github.com/QaidVoid/errand/commit/6c6459eef74a))

### Fixes
- (config) Let the configured model name its provider ([5700e87](https://github.com/QaidVoid/errand/commit/5700e877b6d4))
- (config) Report the search that actually happened ([990d6c5](https://github.com/QaidVoid/errand/commit/990d6c56c010))
- (daemon) Answer where the question was asked, not the channel ([4d9da77](https://github.com/QaidVoid/errand/commit/4d9da77dc296))
- (pr) Open a pull request on the bot's own repository ([a447f6f](https://github.com/QaidVoid/errand/commit/a447f6fd88a5))
- (provider) Skip a malformed model store entry, not the daemon ([d6c8f89](https://github.com/QaidVoid/errand/commit/d6c8f8948fcf))
- (session) Report the model a switch set, not the last turn's ([01fef5d](https://github.com/QaidVoid/errand/commit/01fef5da3c2e))
- (session) Say a thinking level apart from the model name ([15cfb74](https://github.com/QaidVoid/errand/commit/15cfb74277c4))
- (session) Match an aliased model without its level ([129251b](https://github.com/QaidVoid/errand/commit/129251b379ac))
- (session) Let a switch name a model on any configured provider ([c280f8d](https://github.com/QaidVoid/errand/commit/c280f8d29c59))

### Build
- Let vitepress put the site where cloudflare looks ([e86c7b3](https://github.com/QaidVoid/errand/commit/e86c7b3389ed))
