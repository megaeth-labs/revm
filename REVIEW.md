# Reviewing a pull request in this fork

This repository is MegaETH's fork of [bluealloy/revm](https://github.com/bluealloy/revm).
`MEGAETH-FORK.md` holds the branch model, the rules and the upstream touch-point table; this file is the reviewer's checklist against them.
It is read by the automated reviewer as well as by people.

## What every PR must satisfy

1. **Superset.** The public API of every published crate stays a superset of the crates.io baseline recorded in `scripts/mega/crates.txt`.
   The `Semver` workflow enforces it; a PR that must break it carries the `api:exception` label, names the consumers it was checked against (`reth`, `alloy-evm`, `revm-inspectors`, the `op-revm` fork), and adds a row to the exception table in `MEGAETH-FORK.md`.
2. **Thin layer.** The fork adds hooks and data, never MegaETH semantics.
   If a change makes a design decision (a price, a limit, a policy), ask for it to move to `mega-evm` and for the fork to expose only the hook it needs.
3. **Touch points.** Every upstream file the PR modifies has a row in the "Upstream touch points" table of `MEGAETH-FORK.md` with a rebase rule that a future rebase can follow without the author.
   The `touch-points` job checks the rows exist; the reviewer checks the rules are followable.
4. **`no_std`.** No unguarded `std::`, no new dependency that enables `std` by default, no allocation in a path that did not allocate before.
   The CI checks both riscv targets; the reviewer checks the intent.
5. **One topic per PR, `mega:` prefix on every commit.** `git log --oneline v112..main` is the fork's changelog and must stay readable.
6. **Tests live next to the hook.** A hook without a test that fails when the hook is removed is not done.

## By kind of change

| Label | Look for |
|---|---|
| `mega:cherry-pick` | The commit message names the upstream commit (`mega: cherry-pick upstream <sha> — <title>`); the diff equals the upstream diff apart from conflicts the message records; nothing MegaETH-specific rides along |
| `mega:hook` | New logic in new files (`megaeth.rs` modules); upstream files get a `mod` line or a call site only; a default implementation keeps existing behaviour byte for byte; serde-skipped fields keep encodings unchanged |
| `mega:vendor` | The vendored crate keeps its upstream version number and a note of the upstream commit it was taken from; its own tests run in CI |
| `mega:ci` | Workflows pin action SHAs; nothing requires a secret the repository does not have; the local equivalent of every job is documented |
| `mega:rebase` | `scripts/mega/base.txt` and the baseline table move together; every touch-point row was re-applied; the semver baseline versions were updated |

## What not to ask for

- Style changes to upstream code. The fork does not reformat or rename upstream code; conflicts on rebase are the cost.
- MegaETH behaviour tests. They belong in `mega-evm`, which pins this fork by tag.
- Documentation under `docs/` or the book. The fork deleted the book workflow; upstream documentation is upstream's.
