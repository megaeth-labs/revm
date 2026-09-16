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
| `archive/*` | Pre-2026 fork branches, read-only. | Nobody |
| `v112`, `v113`, ... | Upstream `v<N>` tags, pushed unchanged by the nightly workflow. The current base is recorded in `scripts/mega/base.txt`. | Bot only |
| `v40.0.3-mega.N` | Fork releases, always on `main`. | Release workflow |

## Rules

1. **Public API is a superset of upstream.**
   Do not change existing signatures, remove items, add trait methods without a default body, add variants to enums that downstream code matches exhaustively, change the meaning of existing accessors, or rename features.
   `reth`, `alloy-evm`, `op-revm` and `revm-inspectors` are compiled against this fork unmodified.
   The `semver` workflow enforces it here; each consumer's own CI builds against the tagged fork.
2. **Thin layer.**
   The fork adds hooks and data; it does not implement MegaETH semantics.
   If a change needs a design decision, it belongs in `mega-evm`.
3. **Cargo versions never change.**
   The `[patch.crates-io]` mechanism only applies when the patched version satisfies the consumer's requirement, and pre-release suffixes do not satisfy `^40.0.3`.
   Releases are identified by git tags only.
4. **Commit prefix `mega:`, one commit per topic.**
   Every fork commit starts with `mega:` so `git log --oneline v112..main` reads as the fork's changelog.
   Group by topic (the CI is one commit, a hook family is one commit); PRs are squash-merged so `main` stays linear.
   New logic goes into new files; an upstream file gets at most a `mod` line or a call site, and every such touch point is listed in the "Upstream touch points" section below.
5. **`no_std` is mandatory.**
   MegaETH runs the engine inside a zkVM.
   Every crate must keep building for `riscv64imac-unknown-none-elf` with `--no-default-features`.
6. **`FORK_TAG` tracks the release.**
   `revm::megaeth::FORK_TAG` in `crates/revm/src/megaeth.rs` is bumped in the release commit.
   Consumers assert against it at compile time.
7. **Consumers pin tagged commits only.**
   `main` is rewritten when the base line moves, so an untagged commit may become unreachable.
   Nothing enforces this mechanically: `FORK_TAG` still holds the previous release's value on an untagged commit, so the compile-time guard below passes there.

## Upstream touch points

Files that both upstream and the fork edit.
Keep this list current; it is the expected conflict set of a rebase.

| File | Fork change | Rebase rule |
|---|---|---|
| `crates/revm/src/lib.rs` | a short `pub mod megaeth;` block after the crate attributes | keep the block |
| `.github/workflows/ci.yml` | fork CI in place of the upstream matrix | take the fork version; copy new upstream jobs by hand if wanted |
| `.github/workflows/{bench,book,pr-audit,release-plz}.yml`, `.github/dependabot.yml` | deleted | keep deleted (`git rm`) |
| `deny.toml` | advisory ignores the fork needs beyond upstream's list | take upstream's list, re-add the fork-only entries; drop an entry once the crate is gone from `Cargo.lock` |
| `Cargo.lock` | dependency bumps that clear cargo-deny advisories | take upstream's lock, rerun `cargo deny check advisories`, bump again if needed |

Everything under `.github/` resolves to the fork side.
A maintainer who wants that automatic can add `.github/** merge=mega` to `.git/info/attributes` and run `git config merge.mega.driver 'cp %B %A'`; deleted files still need a `git rm`.
Enable `git rerere` so a resolution is only typed once.

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

A consumer that uses fork-only API should add a compile-time guard:

```rust
/// If this fails to compile, the workspace resolves `revm` from crates.io.
/// Copy the `[patch.crates-io]` block from MEGAETH-FORK.md into the root Cargo.toml.
const EXPECTED_REVM_FORK: &str = "v40.0.3-mega.1";
const _: () = assert!(
    str_eq(revm::megaeth::FORK_TAG, EXPECTED_REVM_FORK),
    "revm fork tag mismatch: refresh the [patch.crates-io] block"
);

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}
```

## Release procedure

1. Merge the changes into `main` through PRs.
2. Open the release PR: bump `FORK_TAG` to the new tag name; nothing else changes.
3. Run the `release` workflow with the tag name.
   It checks the tag format, that `FORK_TAG` matches, tags the commit and publishes a GitHub release with the fork changelog (`git log v<base>..HEAD`).
4. Consumers bump `rev` in their patch block and their `EXPECTED_REVM_FORK` in one PR.

## Repository settings (admin)

- Default branch `main`; branch protection: pull request required, status checks `ci success` and `semver-checks` required, force-push allowed for admins only (the base-line move needs it).
- Tag ruleset for `v*-mega.*` that restricts update and deletion only, not creation: the release workflow creates those tags with the Actions token, which cannot bypass a creation restriction. No protection on upstream `v<N>` tags; the nightly workflow pushes them with the same token.
- Actions enabled.
- Repository description points at this file.

## Moving to a new upstream base

Only when `mega-reth` moves its revm line.
This is the one operation that rewrites `main`, and only an admin runs it.

```bash
git fetch upstream --tags
git branch release/v40 v40.0.3-mega.N          # keep the old line reachable
git rebase --onto v113 v112 main               # replay the mega: commits
echo v113 > scripts/mega/base.txt              # and update the crate table versions
# resolve conflicts, run the CI jobs locally, check a consumer against the result (see Consumer wiring)
git push --force-with-lease origin main v113 release/v40
```

Then release `v41.0.0-mega.1`.
Consumers follow together with the `mega-reth` upgrade that triggered the move.

## CI

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | PR, push to `main` | Stable test matrix (three feature sets), `no_std` targets, feature checks, clippy, docs, doctest, fmt, deny, EEST release on x86_64 |
| `semver.yml` | PR, push to `main`, merge queue | `cargo semver-checks` of the twelve crates against their crates.io baseline; any major-level change fails. Required for releases, so keep the push trigger |
| `nightly.yml` | daily | Upstream digest, fast-forward `upstream-main`, mirror upstream tags, full EEST including legacy tests, the `ethtests` profile and i686, `cargo deny` advisories |
| `release.yml` | manual, `main` only | Require green `ci success` and `semver-checks` for the commit, check `FORK_TAG`, tag and publish a fork release |
