# MegaETH fork of revm

This repository is MegaETH's fork of [bluealloy/revm](https://github.com/bluealloy/revm).
It is the execution base of the new `mega-evm` engine.
Crate names stay `revm-*`; consumers switch to the fork through a `[patch.crates-io]` block, never through a renamed dependency.

## Why a fork

revm does not expose the hooks MegaETH needs (multi-dimensional gas, gas preserved on halt, read-only journal access).
Without them, `mega-evm` had to wrap every opcode and copy upstream functions.
The fork adds those hooks as a thin layer; all MegaETH logic stays in `mega-evm`.

## Baseline

| What | Value |
|---|---|
| Upstream tag | `v112` (commit `bf2ce7197ad072a6f5f7416be07813ac99510b93`) |
| Crate versions | `revm 40.0.3`, `revm-handler 20.0.3`, `revm-interpreter 37.0.3`, `revm-context 18.0.3`, `revm-context-interface 19.0.3`, `revm-primitives 24.0.1`, `revm-bytecode 11.0.1`, `revm-state 12.0.1`, `revm-database 15.0.2`, `revm-database-interface 12.1.1`, `revm-inspector 21.0.3`, `revm-precompile 36.0.3` |
| Verified | The `src/` trees of the twelve published crates.io packages are byte-identical to `v112`; every package's `.cargo_vcs_info.json` points at that commit |

The baseline is pinned by `mega-reth`, which locks these exact versions.
It moves only when `mega-reth` moves its revm line.

## Branch model

| Ref | Content | Who writes |
|---|---|---|
| `main` | Upstream history up to the baseline tag, then MegaETH commits on top, linear. `git log v112..main` is the whole fork diff. | PRs, CI green required |
| `upstream-main` | Mirror of `bluealloy/revm` `main`. Fast-forwarded by the nightly workflow. Base for rebases and the upstream digest. | Bot only |
| `release/v<N>` | Maintenance branch for an old base line, created from its last tag when `main` moves to a new base and a backport is needed. | PRs |
| `refs/archive/<branch>` | The pre-2026 fork branches and the old bot branches, moved out of the branch list on 2026-09-16 so that only the live branches show. Not fetched by default; `git fetch origin '+refs/archive/*:refs/archive/*'` brings them back. | Nobody |
| `v40.0.3-mega.N` | Fork releases, always on `main`. The only tags in this repository: upstream tags are not mirrored; `scripts/mega/base.txt` names the upstream tag the fork is based on, and the workflows fetch it from upstream when they need it. | Release workflow |

## Rules

1. **Public API is a superset of upstream.**
   Do not change existing signatures, remove items, add trait methods without a default body, add variants to enums that downstream code matches exhaustively, change the meaning of existing accessors, or rename features.
   `reth`, `alloy-evm` and `revm-inspectors` are compiled against this fork unmodified; `op-revm` is forked separately at [megaeth-labs/op-revm](https://github.com/megaeth-labs/op-revm) and ported there.
   The `Semver` workflow checks every pull request against its base; a deviation passes only with the `api:exception` label, and the pull request body lists every deviating item with the consumers it was checked against.
   Each consumer's own CI builds against the tagged fork.
2. **Thin layer.**
   The fork adds hooks and data; it does not implement MegaETH semantics.
   If a change needs a design decision, it belongs in `mega-evm`.
3. **Cargo versions never change.**
   The `[patch.crates-io]` mechanism only applies when the patched version satisfies the consumer's requirement, and pre-release suffixes do not satisfy `^40.0.3`.
   Releases are identified by git tags only.
4. **Conventional commit prefixes, one commit per topic.**
   Fork commits and pull request titles use the usual prefixes (`feat`, `fix`, `chore`, `docs`, `ci`, `test`) that say what changed; `git log --oneline v112..main` lists only fork commits and reads as the fork's changelog.
   A cherry-pick is titled `chore: cherry-pick upstream <sha> — <title>`.
   Group by topic (the CI is one commit, a hook family is one commit); PRs are squash-merged so `main` stays linear.
   New logic goes into new files where practical; an upstream file gets a `mod` line or a call site.
   `git diff v112..main` is the fork's footprint on upstream; a rebase replays the fork commits and shows the conflicts.
5. **`no_std` is mandatory.**
   MegaETH runs the engine inside a zkVM.
   Every crate must keep building for `riscv64imac-unknown-none-elf` with `--no-default-features`.
6. **The toolchain is pinned.**
   `rust-toolchain.toml` names the compiler this repository is checked with, locally and in CI.
   Upstream floats on `stable` and fixes each release's new clippy lints in its own code; the fork carries upstream code it does not edit, so a floating toolchain would fail CI on lints in code the fork must not touch.
   The pin moves with each base-line move, or in a commit of its own.
   The MSRV consumers see is still `rust-version` in `Cargo.toml`.
7. **Consumers pin tagged commits only.**
   `main` is rewritten when the base line moves, so an untagged commit may become unreachable.
   Nothing enforces this mechanically; the consumer's `Cargo.lock` is the record of what it built against.

## Labels

Every pull request carries exactly one `mega:` label and one `api:` label; the `PR Labels` workflow refuses to merge without them and the Claude label check judges whether they fit the diff.

| Label | Meaning |
|---|---|
| `mega:cherry-pick` | Upstream commits brought into the fork unchanged apart from recorded conflicts |
| `mega:hook` | A hook or data field added in the thin layer |
| `mega:vendor` | A vendored crate (`crates/op-revm`) added or updated |
| `mega:ci` | Workflows, `scripts/mega`, the fork documents |
| `mega:rebase` | The base line moves to a new upstream tag |
| `api:superset` | The public API stays a superset of the baseline (the default) |
| `api:exception` | A registered exception to rule 1; the PR adds a row to the table below |

## Consumer wiring

Consumers declare the crates.io version and redirect all twelve crates in their root `Cargo.toml`:

```toml
[dependencies]
revm = { version = "=40.0.3", default-features = false }

[patch.crates-io]
# megaeth fork of revm, tag v40.0.3-mega.1
revm                    = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-primitives         = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-interpreter        = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-context            = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-context-interface  = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-handler            = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-database           = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-database-interface = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-state              = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-bytecode           = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-precompile         = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
revm-inspector          = { git = "https://github.com/megaeth-labs/revm", rev = "<sha>" }
```

Patch all twelve crates together.
Patching only `revm` pulls the fork's `revm-interpreter` through its path dependency while other consumers still resolve the crates.io one, and the build ends up with two copies.
The block only affects `^40` requirements; a `revm 27` line in the same workspace is untouched.

To check a consumer checkout against a local fork checkout without editing its manifest, pass the patch entries on the command line.
Both paths must be absolute, and the helper's exit status must be checked, otherwise a failed helper leaves `cargo` silently building against crates.io:

```bash
cd /abs/consumer
args=$(/abs/fork/scripts/mega/patch-args.sh /abs/fork) && cargo check --workspace $args
# every fork crate must resolve to exactly one copy of its major
cargo tree --workspace --duplicates $args | grep -cE '^revm v40\.' # expect 0 or 1
```

A consumer that uses `op-revm` takes the MegaETH fork of it as well, pinned to a tag of [megaeth-labs/op-revm](https://github.com/megaeth-labs/op-revm) and redirected at the source the workspace declares it from:

```toml
[patch."https://github.com/ethereum-optimism/optimism"]
# mega-reth declares op-revm from the OP monorepo
op-revm = { git = "https://github.com/megaeth-labs/op-revm", tag = "v20.0.0-mega.1" }
```

That fork names the commit of this repository it is built against in its `.cargo/config.toml`; the consumer's revm patch block points at that commit or a later compatible one.

## Release procedure

1. Merge the changes into `main` through PRs.
2. Run the `release` workflow with the tag name.
   It checks the tag format, tags the commit and publishes a GitHub release with the fork changelog (`git log v<base>..HEAD`).
3. Consumers bump `rev` in their patch block.

## Repository settings (admin)

- Default branch `main`; branch protection: pull request required, status checks `ci success` and `semver-checks` required, force-push allowed for admins only (the base-line move needs it).
- Tag ruleset for `v*-mega.*` that restricts update and deletion only, not creation: the release workflow creates those tags with the Actions token, which cannot bypass a creation restriction.
- Actions enabled; status checks `ci success`, `semver-checks` and `require-labels` required.
- Repository description points at this file.
- Secrets and variables the review bots need: repository secret `CLAUDE_CODE_OAUTH_TOKEN`; the organisation secret `MEGA_MAXWELL_PK` and variable `MEGA_MAXWELL_CLIENT_ID` granted to this repository; the `mega-maxwell` GitHub App installed on this repository (it is the identity the PR reviewer resolves threads and the release workflows push under).
- The Codex reviewer (`chatgpt-codex-connector`) is an organisation-level app; this repository must be added to its repository access in the ChatGPT settings, nothing in the repository configures it.
- Labels from the table above created with `gh label create`.

## Moving to a new upstream base

Only when `mega-reth` moves its revm line.
This is the one operation that rewrites `main`, and only an admin runs it.

```bash
git fetch upstream --tags
git branch release/v40 v40.0.3-mega.N          # keep the old line reachable
git rebase --onto v113 v112 main               # replay the fork commits
echo v113 > scripts/mega/base.txt              # and update the crate table versions
# bump the channel in rust-toolchain.toml to the stable upstream's CI used at that tag
# resolve conflicts, run the CI jobs locally, check a consumer against the result (see Consumer wiring)
git push --force-with-lease origin main release/v40   # upstream tags stay upstream
```

Then release `v41.0.0-mega.1`.
Consumers follow together with the `mega-reth` upgrade that triggered the move.

## CI

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | PR, push to `main` | Test matrix (three feature sets) on the pinned toolchain, `no_std` targets, feature checks, clippy, docs, doctest, fmt, deny, EEST release on x86_64 |
| `semver.yml` | PR | `cargo semver-checks` of the twelve crates against the pull request base; a major-level change fails unless the PR carries `api:exception` (and lists the items in its body) |
| `nightly.yml` | daily | Upstream digest (core-crate commits and upstream tags cut since the last run, posted to the "Upstream digest" issue), fast-forward `upstream-main`, full EEST including legacy tests, the `ethtests` profile and i686, `cargo deny` advisories |
| `release.yml` | manual, `main` only | Require green `ci success` for the commit, tag and publish a fork release |
| `claude.yml` | PR, comments, issues | The shared MegaETH Claude actions: incremental PR review under the `mega-maxwell` identity (reads this file and `REVIEW.md`), label check, issue triage, `@claude` interactive handler |
| `pr-labels.yml` | PR | Exactly one `mega:` and one `api:` label |
