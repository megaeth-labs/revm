# Reviewing a pull request in this fork

This repository is MegaETH's fork of [bluealloy/revm](https://github.com/bluealloy/revm).
`MEGAETH-FORK.md` holds the branch model and the rules; this file is the reviewer's checklist against them.
It is read by the automated reviewer as well as by people.

## What every PR must satisfy

1. **Superset.** The public API of every published crate stays a superset of the crates.io baseline recorded in `scripts/mega/crates.txt`.
   The `Semver` workflow checks API changes against the pull request base; reviewers check baseline compatibility and the documented exceptions.
   A PR that must break the superset carries the `api:exception` label and lists in its body every deviating item with the consumers it was checked against (`reth`, `alloy-evm`, `revm-inspectors`, the `op-revm` fork).
2. **Thin layer.** The fork adds hooks and data, never MegaETH semantics.
   If a change makes a design decision (a price, a limit, a policy), ask for it to move to `mega-evm` and for the fork to expose only the hook it needs.
3. **`no_std`.** No unguarded `std::`, no new dependency that enables `std` by default, no allocation in a path that did not allocate before.
   The CI checks both riscv targets; the reviewer checks the intent.
4. **One topic per PR, conventional prefix on every fork-authored commit.** Fork-authored commits and PR titles start with `feat`, `fix`, `chore`, `docs`, `ci` or `test` and say what changed; cherry-picked commits keep upstream's subject (`MEGAETH-FORK.md` rule 4); `git log --oneline <base>..main` (`<base>` from `scripts/mega/base.txt`) is the fork's changelog and must stay readable.
5. **Tests live next to the hook.** A hook without a test that fails when the hook is removed is not done.

## By kind of change

| Label | Look for |
|---|---|
| `mega:cherry-pick` | The title names the upstream commit (`chore: cherry-pick upstream <sha> — <title>`) or, for a batch, the upstream pull requests (`chore: cherry-pick upstream fixes #a #b …`); branch commits keep upstream's subject; the body links the `Sweep <tag>` issue, lists the `-x` trailers and, for a performance pick, cites the upstream benchmark reference; the diff equals the upstream diff apart from conflicts the message records; nothing MegaETH-specific rides along |
| `mega:hook` | New logic in new files where practical; upstream files get a `mod` line or a call site; a default implementation keeps existing behaviour byte for byte; a new serialized field carries `serde(default)` so payloads that predate it still decode |
| `mega:vendor` | The vendored crate keeps its upstream version number and a note of the upstream commit it was taken from; its own tests run in CI |
| `mega:ci` | Workflows pin action SHAs; nothing requires a secret the repository does not have; the local equivalent of every job is documented |
| `mega:rebase` | `scripts/mega/base.txt`, `scripts/mega/crates.txt`, `rust-toolchain.toml` and the baseline table move together; the upstream tag comes in as a merge commit, not a rebase; the merged tree is compared with the new tag and carried picks reconciled; the PR carries `api:exception` for upstream's own API changes |

## What not to ask for

- Style changes to upstream code. The fork does not reformat or rename upstream code; conflicts at the base-line merge are the cost.
- MegaETH behaviour tests. They belong in `mega-evm`, which pins this fork by tag.
- Documentation under `docs/` or the book. The fork deleted the book workflow; upstream documentation is upstream's.
