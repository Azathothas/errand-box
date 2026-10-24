# errand unchanged, behind bailey

A settled decision of [TODO/RULES.md](../../TODO/RULES.md) section 4, from
2026-10-23 to 2026-10-24, kept here word for word.

## The wording, as it stood

```text
- **errand stays unchanged; errand-box answers as `bailey`.** 2026-10-23. The
  podman backend of errand runs host processes, not a system;
  [`docs/bailey-contract.md`](../docs/bailey-contract.md) says why the
  bailey interface is the seam.
```

## Why it changed

On 2026-10-24 the operator ruled that errand-box takes errand into this tree
as a hard fork: "vendor & patch errand here, hard fork, we will probably
extend/improve it even more". The research of that day is in
[references/2026-10-24-errand-code.md](references/2026-10-24-errand-code.md)
and [references/2026-10-24-errand-tracker.md](references/2026-10-24-errand-tracker.md).
Two findings decided it: errand's own Incus backend, as its maintainer ruled
on errand#41, puts the errand account in `incus-admin`; and a credential
defect in errand's broker (T-029) cannot be fixed from outside errand.

The `bailey` stand-in stays until T-019's native backend replaces it, so
[../bailey-contract.md](../bailey-contract.md) still describes running code.
