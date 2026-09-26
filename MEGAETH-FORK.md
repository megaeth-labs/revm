# MegaETH fork of revm

This repository is MegaETH's fork of [bluealloy/revm](https://github.com/bluealloy/revm).
It is the execution base of the `mega-evm` engine.
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

The baseline moves only when the MegaETH node moves to a newer revm major (see Tracking upstream).

## Branch model

| Ref | Content | Who writes |
|---|---|---|
| `main` | Upstream history up to the baseline tag, then MegaETH commits on top, linear between base-line moves. The log records the fork's history; `git diff <base>..main` shows the current fork footprint (`<base>` is the tag in `scripts/mega/base.txt`). | Squash-merged PRs with one approval and green CI |
| `release/v<N>` | Maintenance branch for an old base line, created from its last tag when `main` moves to a new base and a backport is needed. | PRs |
| `<name>/<type>/<topic>` | Pull request branches; `<type>` is one of `feat`, `fix`, `refactor`, `upgrade`, `doc`, `ci`, `chore`, `spike`. | Contributors |
| `refs/archive/<branch>` | Retired branches, outside the default fetch. | Nobody |
| `v<revm>-mega.N` | Fork releases, always on `main`, never moved or deleted. The only tags in this repository: the workflows fetch the upstream tag named in `scripts/mega/base.txt` from upstream. | Release workflow |

## Rules

1. **Public API is a superset of upstream.**
   Do not change existing signatures, remove items, add trait methods without a default body, add variants to enums that downstream code matches exhaustively, change the meaning of existing accessors, or rename features.
   `reth`, `alloy-evm` and `revm-inspectors` are compiled against this fork unmodified; `op-revm` is forked separately at [megaeth-labs/op-revm](https://github.com/megaeth-labs/op-revm) and ported there.
   The `Semver` workflow checks every pull request against its base; a deviation passes only with the `api:exception` label, and the pull request body lists every deviating item with the consumers it was checked against.
   Each consumer's own CI builds against the tagged fork.
2. **Thin layer.**
   The fork adds hooks and data; it does not implement MegaETH semantics.
   If a change needs a design decision, it belongs in `mega-evm`.
3. **Cargo versions stay fixed between base-line moves.**
   The `[patch.crates-io]` mechanism only applies when the patched version satisfies the consumer's requirement, and pre-release suffixes do not satisfy `^40.0.3`.
   Releases are identified by git tags only.
4. **Conventional commit prefixes, one commit per topic.**
   Fork-authored commits and pull request titles use the usual prefixes (`feat`, `fix`, `chore`, `docs`, `ci`, `test`) that say what changed; `git log --oneline <base>..main` lists only fork commits and reads as the fork's changelog (`<base>` is the tag in `scripts/mega/base.txt`).
   A pull request with one upstream pick is titled `chore: cherry-pick upstream <sha> — <title>`, a batch `chore: cherry-pick upstream fixes #a #b …`; the cherry-picked branch commits keep upstream's subject and the `git cherry-pick -x` trailer.
   Group by topic (the CI is one commit, a hook family is one commit); PRs are squash-merged so `main` stays linear between base-line moves.
   The squash message carries the pull request body.
   New logic goes into new files where practical; an upstream file gets a `mod` line or a call site.
   `git diff <base>..main` is the fork's footprint on upstream.
5. **`no_std` is mandatory.**
   MegaETH runs the engine inside a zkVM.
   Every crate must keep building for `riscv64imac-unknown-none-elf` with `--no-default-features`.
6. **The toolchain is pinned.**
   `rust-toolchain.toml` names the compiler this repository is checked with, locally and in CI.
   Upstream floats on `stable` and fixes each release's new clippy lints in its own code; the fork carries upstream code it does not edit, so a floating toolchain would fail CI on lints in code the fork must not touch.
   The pin moves with each base-line move, or in a commit of its own.
   The MSRV consumers see is still `rust-version` in `Cargo.toml`.
7. **Consumers pin tagged commits only.**
   A tag is the only identity a fork release has (rule 3), and the release workflow tags only commits whose `ci success` is green.
   Consumer tag pinning is not enforced mechanically; each consumer's `Cargo.lock` records the commit it built against.

## Hooks

Each entry names the API a consumer uses and what the fork guarantees about it.
A hook's default keeps upstream's behaviour: a consumer that does not use it sees the same results.
Each entry's **Default** says what the hook costs a consumer that does not use it.

### Code-deposit admission

- **API.** `ContextTr::admit_code_deposit(&mut self, deposit: &CodeDeposit<'_>) -> Result<(), Bytes>`, whose default admits.
  `CodeDeposit` carries `address`, `code`, `gas_before` and `gas_after`; `regular_gas`, `state_gas` and `history_gas` read the deposit's charges off the two trackers.
  It is `#[non_exhaustive]` and built with `CodeDeposit::new`, so a later field is not a breaking change.
- **Contract.** `return_create` calls it once for every creation it deploys, on the plain and the inspected path: after the code passed every check and every deposit charge is recorded, before the journal checkpoint is committed.
  A creation `return_create` fails itself, out of gas on a deposit charge included, is never offered.
  Before Homestead a deposit the frame cannot pay does not fail: `return_create` deploys empty code instead, and offers that deposit with empty code and no charge.
  If the failed charge was a crossing of the frame's withheld gas, `gas_after` carries its crossing record and `gas_before` does not, so an admitted deposit keeps the record and a refusal discards it.
  `Err(output)` refuses: the checkpoint is reverted, the frame's gas is put back to `gas_before`, and the creation ends as a `Revert` with `output`, settling as a creation whose init code reverted.
  A hook that fails fatally records the cause in the context error and refuses; the transaction returns the error.
  The deposit's charges, and with them its state-gas price lookup through `Host::state_gas_price`, are made before it is offered, so that lookup is made whether or not the deposit is admitted; a refusal does not undo what the lookup recorded.
- **Reach.** The hook is on the context `return_create` is called with, so a consumer that wraps `EthFrame`, runs `process_next_action` from its own `frame_run`, or customises `Handler` reaches it by implementing `ContextTr` for its context.
  A context that wraps another and implements `ContextTr` by hand must forward the method, or it gets the admitting default.
- **Default.** `return_create` compiles to the same code, on its own and inlined into the handler on the plain and the inspected path.
  Inlined into a consumer's own function, the admitting default may shift its block layout; where measured, that cost one unconditional branch on one out-of-gas path and changed no success path.

## Tracking upstream

The fork keeps its crate versions at the revm version the MegaETH node pins, today `40.0.3`; a newer upstream tag would bump every crate's major, so the fork takes single upstream commits until the node moves.

Taken: changes to the EIP-8037 / EIP-2780 gas core whatever their prefix; fixes that change execution results (gas, state, logs, halt reasons, precompile output, bytecode analysis, serialized formats); correctness and safety fixes relevant to the consumers, including undefined behaviour, missing out-of-gas checks, panics, and dependency or build failures; performance improvements to execution, precompiles and the database; each with the test-fixture bump that comes with it.
Not taken: tooling-only changes, feature-gated additions the consumers do not use, and change/revert pairs, after checking that no taken commit depends on them.
Picks are batched by crate or subsystem, one pull request per batch, so one reviewer can read a whole batch.
The fork has no benchmark gate: a performance pick must leave execution results unchanged under the crate tests and the EEST run, and cite upstream's benchmark numbers in the pull request body.
A pick that touches a precompile is tested under mutually exclusive feature sets, because `--all-features` selects only the preferred backend in each `cfg_if!` block: `--no-default-features --features std` (every fallback: the arkworks BLS12-381 and KZG paths, `aurora-engine-modexp`, `k256`, `p256`), the default set, `--all-features`, and the single feature that selects the backend the pick changes (`std,blst`, `std,c-kzg`, `std,gmp`, `std,secp256k1` or `std,p256-aws-lc-rs`) on its own.
A pick that breaks the public API needs `api:exception`, the deviating items listed in the pull request body, and a check of every consumer that uses them.
An upstream change that upstream later reverted but the fork still needs lands as a fork-owned opt-in switch (`mega:hook`) whose default keeps upstream's behaviour; the system-call state-gas margin (#55) is the first.
Every pick is applied verbatim: `git show -U0 <commit> | git patch-id --verbatim` gives the same id for the pick and its upstream commit unless the pick's message records a conflict resolution; the pull request body, which becomes the squash message, lists each pick's `(cherry picked from commit <sha>)` line and its conflicts.

For every upstream tag, a maintainer sweeps `<last swept tag>..<tag>` across the whole repository, fixtures, manifests and lockfile included, and records each commit as taken, candidate (with an owner and a disposition) or excluded (with the reason), with both tag SHAs, in the issue titled `Sweep <tag>` that the nightly workflow opens when the tag appears.
The pull request that carries the picks links that issue; a sweep that takes nothing is closed with its classification in the issue.
Before a release, every tag since the last recorded sweep is swept and the release notes cite the sweep issues.

When the node moves to a newer revm major, the fork merges the matching upstream tag into `main` in a pull request labelled `mega:rebase` and `api:exception`, whose body itemises upstream's own API changes.
`scripts/mega/base.txt`, `scripts/mega/crates.txt`, `rust-toolchain.toml` and the Baseline table move in that pull request, and every carried pick and upstream revert is reconciled against the new tag.
The pull request records a green `ci success`, a passing nightly run and a consumer build (see Consumer wiring) on the final merge commit; then, before anything else merges, an admin pushes that merge commit to `main` as a fast-forward, since force-push is refused on every branch.
`v<revm>-mega.1` is released, and the node and its stateless validator move their pins in one change.

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
| `api:exception` | An exception to rule 1; the pull request body lists each deviating item and the consumers checked |

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
The block only affects `^40` requirements; an older revm major in the same workspace is untouched.

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
# the source the workspace declares op-revm from
op-revm = { git = "https://github.com/megaeth-labs/op-revm", tag = "v20.0.0-mega.1" }
```

That fork names the commit of this repository it is built against in its `.cargo/config.toml`; the consumer's revm patch block points at that commit or a later compatible one.

## Release procedure

1. Merge the changes into `main` through PRs; the reviewed base-line merge commit is pushed by an admin, as described under Tracking upstream.
2. Run the `release` workflow with the tag name.
   It checks the tag format, tags the commit and publishes a GitHub release with the fork changelog (`git log v<base>..HEAD`).
3. Consumers bump `rev` in their patch block.

## CI

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | PR, push to `main` | Test matrix (three feature sets) on the pinned toolchain, `no_std` targets, feature checks, clippy, docs, doctest, fmt, deny, EEST release on x86_64 |
| `semver.yml` | PR | `cargo semver-checks` of the twelve crates against the pull request base; a major-level change fails unless the PR carries `api:exception` (and lists the items in its body) |
| `nightly.yml` | daily | Upstream digest (commits to the twelve published crates and upstream tags of the last 25 hours) in the job summary and posted to the issue titled `Upstream digest`, a `Sweep <tag>` issue opened for each new upstream tag, full EEST including legacy tests, the `ethtests` profile and i686, `cargo deny` advisories |
| `release.yml` | manual, `main` only | Require green `ci success` for the commit, tag and publish a fork release whose notes cite the `Sweep <tag>` issues closed since the previous fork tag |
| `claude.yml` | PR, comments, issues | Automated PR review (reads this file and `REVIEW.md`), label check, issue triage, `@claude` interactive handler |
| `pr-labels.yml` | PR | Exactly one `mega:` and one `api:` label |
