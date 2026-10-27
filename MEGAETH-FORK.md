# MegaETH fork of revm

This repository is MegaETH's fork of [bluealloy/revm](https://github.com/bluealloy/revm).
It is the execution base of the `mega-evm` engine.
Crate names stay `revm-*`; consumers switch to the fork through a `[patch.crates-io]` block, never through a renamed dependency.

## Why a fork

revm does not expose the hooks MegaETH needs (multi-dimensional gas, gas preserved on halt, read-only journal access).
Without them, `mega-evm` had to wrap every opcode and copy upstream functions.
The fork adds those hooks, and the MegaETH accounting it carries today inside revm's forwarding and settlement, a set rule 2 names and bounds; every other MegaETH rule stays in `mega-evm`.

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
   The fork carries accepted exceptions, and each entry under Hooks names its own.
   One changes the signature of an upstream function the fork picked with the EIP-8037 / EIP-2780 gas core: `pre_execution::apply_auth_list_eip2780`.
   The fork's takes the context, the authorization list as `&[Eip7702AuthFacts]` and whether EIP-8037 is on, where upstream's takes the journal, an iterator of authorizations and the two state-gas prices, so that every authorization's charge is priced through `Host::state_gas_price` (see State-gas price).
   The base line has no such function, so the change breaks no code written for it.
   A consumer that runs the authorization phase through `pre_execution::apply_eip7702_auth_list` or `Handler::apply_eip7702_auth_list`, whose signatures are upstream's, is unaffected.
   It affects code that calls `apply_auth_list_eip2780` itself, written for upstream's signature: it does not compile against the fork until it passes the context and the list `Eip7702AuthFacts::collect` builds.
2. **Hooks and data, and a bounded set of MegaETH accounting.**
   The fork adds hooks and data.
   It also carries a named, bounded set of MegaETH accounting inside revm's charges and settlement:
   - withheld regular gas and its crossing record, which every regular charge reads;
   - the history-gas dimension, charged reservoir-first, unwound with state gas, and charged for deployed code inside `return_create`.

   Each item has its entry under Hooks, whose Default keeps upstream's results for a consumer that sets nothing.
   Nothing else that implements MegaETH semantics enters the fork.
   What a consumer decides per frame start, such as a budget, a charge the frame's failure gives back, an allowance or a part of the caller's reservoir it keeps, it keeps beside revm's gas, driven by the start lifecycle's hooks; the lifecycle carries no MegaETH figure, so it is a generic hook and the set does not grow with it (see Start lifecycle).
   An addition to the set needs all of these, argued in its pull request:
   - no hook that sees only the state revm exposes at that point can make the same decision, because the figure is read or written inside a charge, or has to be ordered with revm's own settlement; the pull request says why the consumer cannot keep it beside revm's gas;
   - it is mechanism, not policy: every amount, share, price and limit comes from the consumer, through the gas schedule or a hook, and a detail the fork has to fix itself, such as a rounding, is named in its entry;
   - zero, unlimited or none as its default leaves every result upstream's, and its entry states what a consumer that sets nothing pays for it, in bytes on the frame results and in instructions;
   - its entry names the condition under which the fork drops it.

   A design decision still belongs in `mega-evm`: the fork carries the figures and moves them with the gas, and `mega-evm` decides what they are and what happens when one runs out.
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

## What the fork carries

`git diff <base>..<fork head>` is the fork's footprint on upstream (rule 4).
It falls into five classes, and the next base-line move (see Tracking upstream) treats each one differently.
The sizes are the lines of `crates/`, without tests, that `git blame <base>..<fork head>` attributes to each class's commits.

| Class | What | Size | At the next base-line move |
|---|---|---|---|
| Upstream gas core | The EIP-8037 / EIP-2780 gas core, picked from later upstream releases | about 2,600 lines | Goes away once the new base contains every pick; the sweep issues record which tag first contains each. The settlement code the fork's MegaETH accounting is written into is upstream's own (`handle_reservoir_remaining_gas`, `rollback_state_gas`, `return_create`), so that accounting is re-applied to the new base's versions of it. |
| Other upstream picks | Fixes and performance picks | about 140 lines | Goes away once the new base contains them; a pick the base lacks is picked again or dropped, as its sweep decides. A behaviour a switch takes back stays switchable (see Consumer switches). |
| Generic hooks | State-gas price, precompile price, code-deposit admission, the failed-charge record, the completed-write and frame-start hooks, the pending action, the frame-run-result hook and the start lifecycle | not counted | Each is proposed upstream, in a form upstream can take: without MegaETH vocabulary in its types, as an instruction-level helper rather than a `Host` method where one serves, and with no cost on the default path that upstream would not accept. It is carried until upstream has an equivalent (its Removal). |
| Consumer switches | Amsterdam opcodes and EIP-7708 below Amsterdam, the base line's journal semantics | about 300 lines | Carried until their Removal conditions. The Amsterdam switch is proposed in the shape of upstream's per-EIP activation switches, which already cover three of its four opcodes; the base line's journal semantics are not proposed. |
| MegaETH accounting (rule 2) | Withheld regular gas, the history-gas dimension | about 700 lines | Carried and re-applied to the new base, or replaced: an item a later design keeps in `mega-evm` instead, with the same results for the consumer that uses it, leaves the fork, and its entry's Removal says when. |

## Hooks

This section lists every hook, switch and accessor the fork carries, and each item of rule 2's MegaETH accounting.
Each entry names the API a consumer uses and what the fork guarantees about it.
A hook's default keeps upstream's behaviour: a consumer that does not use it sees the same results.
Each entry's **Consumer** says who uses it and why the fork carries it, its **Removal** the condition under which the fork drops it, and its **Default** what it costs a consumer that does not use it.

A field a hook adds to a serialized type carries `serde(default)`, so in a format that carries field names, such as JSON, an encoding made before the field existed decodes with the default.
A format that encodes a struct by position, such as bincode, cannot read such an encoding, as with upstream's own field additions: it reads the old fields in their places and then looks for the new ones, so a value persisted in such a format must not cross an upgrade of the fork that adds a field.

A gas schedule (`GasParams`) is a table of 256 entries, not named fields, so `serde(default)` cannot reach an entry: a table built raw (`GasParams::new`) or decoded from an encoding made before an entry existed holds zero there.
Every entry the fork adds is therefore zero by default, and zero is upstream's behaviour: such a table behaves as the default schedule does.
An id the fork once used for an entry is left unused rather than given a new meaning, so a table encoded then does not read as a setting: ids 246 to 254 are such ids.

### State-gas price

- **API.** `Host::state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64>` and `Host::state_gas_charge(&mut self, charge: StateGasCharge) -> Option<u64>`.
  The value types `StateGasSite` (`account`, `slot`) and `StateGasCharge` (`one`, `units`, `total`) in `revm_context_interface::cfg`.
  For the refund: `charged_state_gas_address` on `CreateInputs`, `CallOutcome` and `CreateOutcome`, and `FrameResult::refundable_state_gas_charge`.
  `pre_execution::Eip7702AuthFacts`, the authorization list lifted out of the transaction so the authorization phase can hand the whole context to the hook, and the signature of `pre_execution::apply_auth_list_eip2780`, which takes the context and that list in place of upstream's journal, authorization iterator and two state-gas prices (an exception to rule 1).
- **Contract.** Every EIP-8037 state-gas charge and every refund of one is priced through `state_gas_price`, so a charge and the refund that undoes it agree by construction.
  `site` names the account the charge lands on, and the storage slot for a slot-scoped charge; a creation is priced at the address it deploys at, the first frame's included.
  `None` reports a failed lookup: the implementation records the cause in the context error, and the charge site turns it into a fatal external error.
- **Consumer.** mega-evm prices state gas by the storage bucket of the account or slot a charge lands on. Only the charge site knows the site, and the price must be the same at the charge and at its refund.
- **Removal.** When upstream prices state gas through an equivalent hook, or when mega-evm prices every state-gas charge from the flat schedule.
- **Default.** Reads the flat schedule (`gas_params().get(id)`) and never fails, so every price is upstream's, and so is every charge and refund but one.
  With EIP-2780 on, the first frame's creation is checked and charged at the address it deploys at, which the caller's account nonce gives, where upstream's derives the address from the transaction's nonce.
  The two agree while the nonce check is on.
  With the nonce check off and a transaction nonce that differs from the caller's account nonce, as for an OP deposit, upstream's checks, charges and warms an address the transaction does not deploy at, and the fork's the address it deploys at.
  There, whether the creation pays `create_state_gas` can differ from upstream's, and so can the gas of a later access to the other address, which upstream's leaves warm and the fork's leaves cold.
  The refund carries one address more in the frame inputs and outcomes, and under EIP-2780 an EIP-7702 transaction's authorization list is buffered before the authorization phase, one 88-byte entry per authorization that can apply.
  `CallOutcome` and `CreateOutcome` are constructible by literal and carry `charged_state_gas_address`, which upstream's do not, so an outcome literal written for upstream's fields does not compile against the fork until it names it (`Address::ZERO` with nothing charged).

### History gas

- **API.** On `GasTracker`, forwarded on `Gas`: `record_history_cost`, `refill_history`, `history_gas_spent`, `set_history_gas_spent`.
  `GasId::code_deposit_history_gas` and `GasParams::code_deposit_history_gas(len)`.
- **Contract.** A history charge draws the same reservoir-first budget as a state-gas charge: the reservoir, then regular gas, withheld part first.
  It is booked on its own counter, so history gas can be reported and limited apart from state gas.
  It unwinds with state gas: a frame that reverts or halts refills both in last-in-first-out order, and `refill_history` returns a spill before it tops up the reservoir.
  `return_create` charges `code_deposit_history_gas` for the deployed bytes after their state-gas charge.
  `ResultGas` has no field for it: `total_gas_spent` and `block_regular_gas_used` include it, and the `ResultGas` documentation says how a block-level consumer separates it.
- **Consumer.** mega-evm charges history gas for the bytes a transaction appends to the chain's history. The code deposit is charged inside `return_create`, and only the tracker can unwind a frame's history charges with its state charges.
- **Removal.** When upstream carries an equivalent gas dimension, or when mega-evm stops charging history gas.
- **Default.** No schedule prices `code_deposit_history_gas`, and nothing else in the workspace charges history gas, so the counter stays zero and every result is upstream's.
  In a format that carries field names, the counter decodes as zero from encodings that predate it.

### Amsterdam opcodes and transfer logs below Amsterdam

- **API.** `revm_interpreter::enable_amsterdam_opcodes(&mut InstructionTable)` and the ungated instructions it installs (`dupn_enabled`, `swapn_enabled`, `exchange_enabled`, `slot_num_enabled`); `DummyHost::with_slot_num`.
  `CfgEnv::enable_amsterdam_eip7708` with `with_enable_amsterdam_eip7708`, `Cfg::enable_amsterdam_eip7708` (default `false`), and `JournalTr::set_amsterdam_eip7708_enabled` (default no-op).
- **Contract.** `enable_amsterdam_opcodes` overwrites `DUPN`, `SWAPN`, `EXCHANGE` and `SLOTNUM` with implementations that do not check the spec, and changes nothing else; the gas table is unchanged, since these four have the same static gas on every spec.
  A consumer that wraps table entries installs first and wraps after.
  With `enable_amsterdam_eip7708` set, the journal emits EIP-7708 transfer logs on any spec; `amsterdam_eip7708_disabled` still wins over both the spec and the switch.
  The journal learns the switch through the context's cfg sync (construction, `with_cfg`, `modify_cfg`, journal or database replacement): assigning the field on an existing context's `cfg` does not reach it, and a custom `JournalTr` that keeps the default no-op emits no logs below Amsterdam.
- **Consumer.** mega-evm runs Amsterdam's opcodes and transfer logs on a spec built on Osaka. Upstream gates both on `SpecId::AMSTERDAM` inside the instructions and the journal.
- **Removal.** When the spec mega-evm runs is built on Amsterdam, or upstream activates these features independently of the spec.
- **Default.** Nothing calls `enable_amsterdam_opcodes`, and the switch is off, so the table and the journal behave as upstream's.

### Withheld regular gas and the crossing record

- **API.** On `GasTracker`, forwarded on `Gas`: `spendable`, `withheld`, `limit_spendable`, `withhold`, `release_withheld`, `record_withheld_first_cost`, `withheld_crossing`, `clear_withheld_crossing`, `set_withheld_crossing`.
  The type `WithheldCrossing` (`with_remaining`, `remaining`), re-exported from `revm_interpreter::gas`.
- **Two parts.** A frame's regular gas is a spendable part and a withheld part; `remaining()` is their sum, and every reader of the frame's gas sees the sum: `GAS`, the `CALL` and `CREATE` forward, the `SSTORE` stipend sentry, the skip-cold-load checks, the gas a child returns, the reimbursement.
- **Charges.** A regular charge draws the spendable part only.
  A deduction that is not the frame's own regular work (the gas forwarded to a child, a state or history charge spilling past the reservoir) draws the withheld part first and fails only when the sum cannot pay.
  Credits land on the spendable part.
- **Crossing.** A regular charge with `spendable < cost <= spendable + withheld` fails as it would with nothing withheld, and leaves a `WithheldCrossing` holding the regular gas left before the charge: `remaining()`, both parts together.
  `set_remaining(crossing.remaining())` puts the frame's regular gas back to what it had before the failed charge, whatever the halt did to it.
  It puts back the amount; the split comes back only as far as the halt left it.
  An `OutOfGas` halt of the interpreter zeroes both parts, so after it the whole amount comes back spendable.
  `return_create`'s own `OutOfGas`, on a code-deposit or code-hash charge that crossed, zeroes nothing, so there the split comes back as it was.
  A charge beyond the sum records no crossing, and neither does an operand refused before any charge; the failed-charge record below covers every failed charge.
  The record is the last crossing since it was cleared, survives `spend_all`, starts empty in every child, and is not taken over by a parent.
  A consumer marks a result it produced itself as a crossing with `set_withheld_crossing`, passing `with_remaining` the regular gas the frame had before the charge that crossed.
  A call it holds to an allowance below its forward and answers without running crossed exactly when `allowance < cost <= forward` for the gas `cost` the call needs (for a precompile, the price `Precompile::required_gas` answers); its record is then `with_remaining(forward)`, and a cost above the forward is a plain out-of-gas with no record.
  A frame whose regular charge fails halts, except before Homestead, where `return_create` deploys empty code when the frame cannot pay the code deposit: a crossing on that charge leaves the record on a creation that returns successfully.
- **Consumer.** mega-evm holds a frame's regular charges to what its gas detention allows. Every regular charge and every reader of the frame's gas is inside the interpreter, so only the tracker can keep part of the gas from the charges while every reader still sees it.
- **Removal.** When upstream carries an equivalent split of a frame's regular gas, or when mega-evm stops holding regular charges below a frame's gas.
- **Default.** With nothing withheld every method behaves as upstream's.
  The record is one word; a crossing is recorded in a cold, out-of-line function on the failure side of a regular charge, so a charge that succeeds runs upstream's code.

### The failed-charge record

- **API.** `GasTracker::failed_charge_remaining() -> Option<u64>` and `GasTracker::set_failed_charge_remaining(Option<u64>)`, both forwarded on `Gas`.
- **Contract.** Every deduction that fails and ends the frame out of gas records the regular gas left before it, spendable and withheld parts together:
  - a regular charge (`record_regular_cost`, `record_cost_unsafe`), whether or not the withheld part would have paid it;
  - a state or history charge whose spill past the reservoir regular gas cannot pay (`record_state_cost`, `record_history_cost`), whether the reservoir paid nothing or part of it; the record is regular gas alone, and the reservoir is left as it was;
  - a forward to a child that regular gas cannot pay (`record_withheld_first_cost`), which only a call before EIP-150 can fail;
  - an out-of-gas halt of the interpreter with no record there, before the halt zeroes the frame's gas: a cold access the instruction refused before its charge because the frame could not pay it (an account read, `SLOAD`, `SSTORE`, `SELFDESTRUCT`, a call's target and the account it delegates to), an out-of-gas a consumer's instruction returned itself, or one an instruction hook asked for (see Completed-write hook and Frame-start hook); a record already there is the failure that caused the halt, and is kept;
  - a precompile run that needs more gas than the frame has: the frame's whole gas limit, as nothing ran.

  The failed deduction took nothing, so the record is all the regular gas the frame had; the unchecked charge records it before the spendable part wraps, and a failure with nothing left records `Some(0)`.
  A deduction that succeeds leaves the record as it is, and so does every credit and setter of the regular gas.
  The fork never clears the record: it survives `spend_all`, a later failed charge replaces it, it starts empty in every child, and a parent does not take over its child's.
  A frame whose charge fails halts, except before Homestead, where `return_create` deploys empty code when the frame cannot pay the code deposit: the record is then left on a creation that returns successfully.
  A consumer that builds an out-of-gas result itself, outside the interpreter's halt, records the same with `set_failed_charge_remaining`.
- **Not recorded.** The other out-of-gas halts zero nothing, so the frame's gas stays readable in its result: an operand refused before any charge, the memory limit, and the `SSTORE` stipend sentry.
  A charge that fails before the first frame, in the transaction's runtime gas phase, records on the transaction's gas, which the handler then rebuilds; no frame ran.
- **Consumer.** mega-evm's gas detention reads what a halting frame burned. An `OutOfGas` halt zeroes the frame's regular gas inside the interpreter, before the frame's result reaches any boundary mega-evm owns.
- **Removal.** When upstream keeps a halting frame's regular gas readable, or when mega-evm no longer reads what a halting frame burned.
- **Default.** Each record is written on the failure side only: the tracker's in cold, out-of-line functions, the interpreter's in its cold out-of-gas halt, the handler's on a precompile's out-of-gas. A deduction or load that succeeds runs the same code.
  It takes two words of the tracker, a number and a flag, and the flag's word also holds the start mark (see Start lifecycle); every copy of the tracker, and of every `Gas` and frame result that holds one, carries both words.
  Encoded, the record is an optional number in the field's place.
  Where measured (callgrind, the fork's release profile), the field left a transfer and a precompile call unchanged and cost about 0.1% of the instructions of a creation; the records written on the failure side add nothing measurable.

### Precompile price

- **API.** `Precompile::required_gas(&self, input: &[u8]) -> Option<u64>`; `Precompile::with_required_gas(self, PrecompileGasFn) -> Self`; `PrecompileGasFn = fn(&[u8]) -> u64`; the built-in price functions in `revm_precompile::required_gas`.
- **Contract.** `Some(price)` guarantees, for every gas limit `g`: the run halts `OutOfGas` if and only if `g < price`, and with `g >= price` its result is the same for every `g`, a success or a revert using exactly `price`.
  A malformed input keeps the contract: refused by a check made before any gas check, it prices at `0`; refused after a gas check, it prices at what that gas check asks for; `MODEXP` header sizes refused between its two gas checks price at the first check's minimum.
  The price does not tell whether a call with enough gas succeeds.
- **Price what is dispatched.** The price belongs to the `Precompile` value a consumer dispatches, and holds only for that value's run.
  A table that type-erases its entries cannot be priced through: alloy-evm's `PrecompilesMap` is one once it holds dynamic precompiles.
  An address a node replaced must not be priced from the built-in set, whose value still answers for it.
  The fork cannot see what a consumer dispatches; this is the consumer's to keep.
- **A held call.** A call a consumer holds to an allowance below its forward crosses the allowance exactly when `allowance < price <= forward`.
  A price above the forward is a plain out-of-gas, although a run on the allowance runs out of gas too; only the price tells the two apart.
  Charging the price on a `GasTracker` that holds the forward, limited to the allowance with `limit_spendable`, records a crossing (`withheld_crossing`) in exactly that case, so the tracker can build the record.
- **Coverage.** Every precompile the crate defines carries its price function, so every precompile of every set `Precompiles::new` returns answers.
  A precompile built with `Precompile::new` carries none and answers `None`.
  A precompile that wraps a built-in must price itself from the price function of the run it wraps, and price its own refusals at `0`; `with_required_gas`'s documentation shows the pattern, and such a wrapper needs a differential test against its own run.
  A precompile defined outside this repository that wraps a revm run must price itself from the run it wraps, as the op-revm fork's size-limited wrappers do; that fork's `MEGAETH-FORK.md` ("Precompile prices") carries their guard.
- **Guard.** `ecrecover`, `sha256`, `ripemd160` and `blake2f` repeat literals their runs keep private; the differential test `required_gas_matches_every_builtin_run`, run under every backend, fails when a run's gas changes and its price does not.
- **Consumer.** mega-evm holds a precompile call to an allowance below its forward and must tell a crossing from a plain out-of-gas without running it. A precompile's gas rule is private to its run.
- **Removal.** When upstream exposes a precompile's price before the run, or when mega-evm stops holding precompile calls to an allowance.
- **Default.** Nothing on the execution path calls it; `Precompile` is one word larger.

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
- **Consumer.** mega-evm refuses a deposit its own rules reject, after revm's checks and charges and before the creation commits, and counts the deployed code against the creating frame's budget there. That point is inside `return_create`, between its last charge and the checkpoint commit.
- **Removal.** When upstream has an equivalent admission point, or when mega-evm stops refusing deposits.
- **Default.** `return_create` compiles to the same code, on its own and inlined into the handler on the plain and the inspected path.
  Inlined into a consumer's own function, the admitting default may shift its block layout; where measured, that cost one unconditional branch on one out-of-gas path and changed no success path.

### revm 40.0.3's journal semantics

- **API.** `CfgEnv::legacy_journal_semantics` with `with_legacy_journal_semantics`, `Cfg::legacy_journal_semantics` (default `false`), and `JournalTr::set_legacy_journal_semantics` (default no-op).
- **Contract.** With the switch set, the journal records what revm 40.0.3 recorded where two picked upstream fixes changed it: a storage slot warmed while cold for the current transaction re-baselines its EIP-2200 `original_value`, and a `CodeChange` entry records `KECCAK_EMPTY` and no code as the previous code, on the journal's path and on the account-level `set_code`.
  The journal learns the switch through the context's cfg sync, as it learns `enable_amsterdam_eip7708`.
- **Consumer.** MegaETH's legacy engine, which replays blocks produced on revm 40.0.3 and must reproduce them exactly. The two behaviours are inside the journal.
- **Removal.** When no consumer needs to reproduce revm 40.0.3's journal.
- **Default.** Off, which is the picked behaviour; the journal reads one `bool` when it warms a cold slot and when it records a code change.

### Completed-write hook

- **API.** `Host::write_completed(&mut self, write: CompletedWrite<'_>, gas: &mut GasTracker) -> WriteVerdict`, whose default continues.
  `CompletedWrite` (`Sstore`, `Log`, `SelfDestruct`, with the write's account and what the instruction knows of it; `SelfDestruct` also carries `balance`, the destructed account's balance before the write) and `WriteVerdict` (`Continue`, `OutOfGas`, `Revert(Bytes)`), in `revm_context_interface::host`.
  `CompletedWrite` is exhaustive on purpose: mega-evm matches it to meter each write, so a new kind is a compile error in every consumer that matches it rather than a write that passes a wildcard arm un-metered, and adding one is an `api:exception` change.
  Its variants' fields are exhaustive too: a consumer that builds a variant, or matches one naming every field without `..`, names every field, `SelfDestruct`'s `balance` included, and a field added to a variant is an `api:exception` change as a new kind is.
- **Contract.** `SSTORE`, `LOG0`–`LOG4` and `SELFDESTRUCT` call it once the write is in the journal and every charge the instruction makes has been made, its refund included, so only on the instruction's success path, on the plain and the inspected path.
  `gas` is the running frame's tracker. The hook may debit and credit it, and the instruction uses it as the hook left it: nothing is charged after the hook. A charge the hook makes that fails records as any failed charge does (see The failed-charge record), whatever the verdict.
  `Continue` ends the instruction as it would have ended. `OutOfGas` fails it as a failed charge does: the frame halts out of gas, and the halt records the regular gas the hook left unless a failed charge already did. `Revert(output)` ends the frame as a `Revert` with `output`, on its gas as the hook left it; a `SELFDESTRUCT` so reverted does not self-destruct.
  Only these instructions call it: a write the journal records any other way (a value transfer and its log, deployed code, an inspector's or a precompile's write) is not offered.
  A destruction changes only balances, no nonce and no code. So with the journal at the hook, `SelfDestruct`'s `balance` rebuilds both accounts as they were before the write: the destructed account held `balance`, and a beneficiary that is another account held its balance less `balance`, since it got all of it.
  When the beneficiary is the destructed account itself, the account's balance after the write says whether the spec burned the balance or left it.
- **Consumer.** mega-evm meters each completed write against its per-transaction limits and charges its history. Without the hook it wraps the three instructions and stages what its `Host` saw.
  It counts an account's write record while the account's info differs from what the transaction loaded. A destruction's two accounts can look the same after the write whether or not they had changed before it (loaded with 5, now 0, from 5 or from 7), so it needs the balance before the write, which only the instruction reads before the write.
- **Removal.** When upstream exposes an equivalent point, or when mega-evm stops metering writes.
- **Default.** Continues and touches nothing. The facts are references to values the instruction holds anyway, except `SelfDestruct`'s `balance`, which the instruction reads for the hook from the running frame's own account just before the write. The journal holds that account warm, so the read loads nothing cold.
  Where measured (callgrind, the fork's release profile), `SSTORE` and `LOG0`–`LOG4` compile to the same code as without the hook. The read costs a `SELFDESTRUCT` 119 instructions, one warm journal lookup (+1.5% of a transaction that self-destructs a contract). It is a `Host::load_account_info_skip_cold_load` call that upstream's instruction does not make, so a `Host` that counts or refuses loads sees one more per `SELFDESTRUCT`.

### Frame-start hook

- **API.** `Host::frame_starting(&mut self, start: &mut FrameStart, gas: &mut GasTracker) -> bool`, whose default returns `true`.
  `FrameStart` (`#[non_exhaustive]`, built with `FrameStart::new`: `kind`, `caller`, `target`, `value`, `forward`, and `reservoir`, which `new` leaves `None`) and `FrameStartKind` (`Call`, `CallCode`, `DelegateCall`, `StaticCall`, `Create`, `Create2`), in `revm_context_interface::host`.
  `FrameStartKind` is exhaustive on purpose, as `CompletedWrite` is: mega-evm matches it to charge each start, so a new kind is a compile error in every consumer that matches it rather than a start that passes a wildcard arm uncharged, and adding one is an `api:exception` change.
- **Contract.** `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2` call it after every charge they make and after the forward is taken from the caller's gas, before they build the frame's input, on the plain and the inspected path.
  `forward` is the regular gas taken: the frame's gas limit without a value call's stipend. `value` is the value transferred, zero for `DELEGATECALL` and `STATICCALL`.
  `target` is the account the frame runs as: the account called for `CALL` and `STATICCALL`, the calling frame's own for `CALLCODE` and `DELEGATECALL`, and for `CREATE` and `CREATE2` the address the creation would take.
  Whether a creation takes it is the outcome's to say (`CreateOutcome::address`): a creation that lands on an account that already has code or a nonce, one the handler refuses after the hook (an inspector changed the caller's balance), and one an inspector's `create` or a consumer's `frame_init` answers without a frame take none, though their start named the address. A consumer that counts the created account from `target` settles that count on the outcome.
  A creation's address is computed once, into the creation's input, and the handler builds the frame at it without computing it again. With EIP-8037 on, the instruction's EIP-8037 branch computes it. With EIP-8037 off, the instruction computes it after the same checks, reading the caller's balance and nonce from the journal.
  It is `None` only for a creation the handler fails before it takes an address: an endowment above the caller's balance, a caller's nonce at its maximum, or a creating frame at the depth limit. Such a creation reaches the hook only with EIP-8037 off; with it on, the instruction ends the creation before the hook.
  The address is fixed when the hook sees it. Something that changes the caller's nonce between the instruction and the frame does not move it, as on upstream's EIP-8037 path. An inspector that changes the creation's caller, scheme or init code (`CreateInputs::set_call`, `set_scheme`, `set_init_code`) clears it, and the handler computes the new one.
  `gas` is the caller's tracker. The hook may debit and credit it, and the instruction uses it as the hook left it: nothing is charged after the hook, and the frame's input copies the reservoir the hook left in it, so a frame the handler runs and one it answers without running (a precompile) both start from that reservoir. A charge the hook makes that fails records as any failed charge does, whatever the answer.
  `start` is the hook's to name the child's reservoir on (`reservoir`), if it takes that out of `gas` itself; the instruction reads nothing else back from it.
  `FrameStart::reservoir` is `None` when the hook is called. `None` leaves the frame's input a copy of the reservoir in `gas`, which the caller keeps while the child runs, as upstream's instructions do. `Some(r)` means the hook took the child's reservoir out of `gas` itself: the frame's input carries `r` instead, and the caller keeps what the hook left it. The instruction does not check `r` against what the hook took.
  Either way the child's settlement replaces the caller's reservoir with the one the child returns, as upstream's does; a consumer that kept part of the caller's reservoir adds it back when revm settles the child (`FrameSettle::held_reservoir`, see Start lifecycle).
  `true` hands the start to the handler, which may still refuse it (call depth, balance) or answer it without a frame; what the hook did to `gas` stands either way. `false` fails the instruction out of gas and starts no frame: the forward stays taken unless the hook gave it back, and the halt records the regular gas the hook left unless a failed charge already did.
  A creation the instruction ends before its forward (EIP-8037's checks of the endowment, the nonce and the depth) is not offered.
- **Consumer.** mega-evm charges a start's write records after the forward and before the frame inherits the caller's reservoir. Without the hook it wraps the call and creation instructions, rewrites the reservoir revm already copied into the input, and takes the input back when the charge fails.
  It counts the created account's record only when the transaction has not changed that account already (funded it, say), so it needs the address at the start. A `CREATE2`'s address depends on the init-code hash, which only the instruction has.
- **Removal.** When upstream exposes an equivalent point, or when mega-evm stops charging frame starts.
- **Default.** Returns `true` and touches nothing.
  Where measured (callgrind, the fork's release profile), a call runs one instruction more than without the hook: the store of the forward its helper returns, about 0.05% of a call to a contract that stops.
  For `CREATE` and `CREATE2` the hook's only cost is the start's address.
  With EIP-8037 off, the default spec's path, the address costs a warm journal read of the caller and the three checks: 149 instructions per `CREATE` and 147 per `CREATE2` from a contract (+0.95% and +0.73%). The read is a `Host::load_account_info_skip_cold_load` call that upstream's instruction does not make, so a `Host` that counts or refuses loads sees one more per creation.
  The instruction computes the address and a `CREATE2`'s init-code hash, and the handler reuses them, so a creation the handler builds or fails computes the same hashes as upstream's.
  A start that never reaches the handler's frame building computes them for the hook alone: one hash for a `CREATE`, two for a `CREATE2`. That is a start the hook refuses, and a creation answered without a frame, by an inspector's `create` or by a consumer's own `frame_init`. Upstream's EIP-8037 path computes them before the hook in the same way.
  With EIP-8037 on, the start reads the address the EIP-8037 branch already computed from the input; that path was not measured.
  `CreateInputs`' derived `PartialEq` compares the cached address and hash, which the input of a creation the instruction starts holds with EIP-8037 on and off, so it compares unequal to an otherwise equal input that has computed nothing, and to its own serde round trip, which skips the cache.
  A consumer tells a creation from a call by `kind`, not by `target`: `target` is `None` only for the creations the Contract names, and for every other start an address, which for a creation is the one it would take.

### The pending action

- **API.** No new item: upstream's `LoopControl::set_action` on a frame's interpreter, which `Interpreter::run_plain` (and `run_plain_as_output`) and `revm_inspector::inspect_instructions` honour before their first instruction, and which `InspectorEvmTr::inspect_frame_init` reads before it hands a new frame to `Inspector::initialize_interp`.
- **Contract.** An action set on a frame's interpreter before the frame runs is what the run returns, before any instruction runs. That is an action set once `frame_init` built the frame, or once `frame_return_result` put a child's result into its caller.
  The plain and the inspected run return it alike: no instruction executes, so the frame's gas, stack, memory and program counter are as they were when it was set, and the inspected run calls neither `Inspector::step` nor `step_end` for that run.
  The handler then processes it as any instruction's action (`process_next_action`): a `NewFrame` starts a child, a `Return` ends the frame on its result, as an instruction's return would.
  A run that starts with no action set runs as upstream's: the inspector's `step` comes before each instruction, and an action the inspector sets in `step` ends the run after that `step`, as upstream's does.
  A frame whose action is set when it is built is not handed to `initialize_interp`, and runs no `step` before that action: the action `frame_init` set is the frame's, which no inspector can replace, and the inspector sees the frame's start and end, as it sees a precompile's answer.
  If that action starts a child and the frame resumes with no action set, the frame runs its code with a `step` before each instruction, never having been handed to `initialize_interp`.
  An action an inspector sets in `initialize_interp` on any other frame is pending when the run starts, so it is returned the same way, without a `step`; upstream's inspected run calls `step` once before returning it, and returns the action as that `step` left it (see Baseline behaviour the fork changes).
  One action at a time: `set_action` asserts in a debug build that none is pending, so a consumer sets an action only on a frame that has none.
- **Consumer.** mega-evm makes some of a frame's actions outside an instruction: the creation and the answer of a system contract's call, and the stop of a frame that a child left over a limit when it returns. They come back through the frame's run, so the loop starts the next frame or ends the frame, and an inspector sees the frame end.
  Without the check, the plain run executes the frame's code before it returns the action, so mega-evm overrides `frame_run`, and the inspected run shows an inspector a `step` of an instruction that never runs.
  Without the `initialize_interp` rule, an inspector that halts the frame of a system contract's call there trips `set_action`'s assertion in a debug build, and in a release build replaces the creation mega-evm set with its halt.
- **Removal.** When upstream's runs check for a pending action before their first instruction and its inspected frame start does not hand a frame with one to `initialize_interp`, or when mega-evm makes no action outside an instruction.
- **Default.** revm sets no action before a run, so the check finds none, every frame revm builds is handed to `initialize_interp`, and every run is upstream's, callbacks included, for a consumer and an inspector that set none.
  Where measured (callgrind, the fork's release profile), the check costs about three instructions per frame run on the plain path: 6.03 per `CALL` (+0.23% on a transaction of 64 calls) and 6.12 per nested `CREATE`, which each run two frames (the child, and the caller's resume); 2 or 3 for a transaction whose one frame runs code; nothing for a transfer to an account without code or a precompile's answer, which run no frame. The inspected path, which also reads the action once per new frame before `initialize_interp`, was not measured.

### Frame-run result hook

- **API.** `EvmTr::frame_run_result(&mut self, result: &mut <Self::Frame as FrameTr>::FrameResult) -> Result<(), ContextDbError<Self::Context>>`, whose default does nothing.
- **Contract.** The handler loops hand it the result of each frame that ran and returns, once, as the frame's run left it: after `process_next_action` processed the frame's last action (`return_create`'s checks and charges included, for a creation), with the frame still on top of the stack.
  Whether the frame is marked finished differs by path and frame type, so the hook does not rely on it: revm's `frame_run` marks it before the plain loop calls the hook, the inspected run marks a frame with an `EthFrame` after the inspector's end callbacks, as upstream's does, and a frame without one is marked by its own `frame_run`, before the hook.
  On the plain path `Handler::run_exec_loop` calls it when `frame_run` returns a result. On the inspected path `InspectorEvmTr::inspect_frame_run` calls it before the inspector's `call_end` or `create_end` and its `frame_end`, and also for a frame without an `EthFrame`, whose run calls no inspector. On both, it runs before `frame_return_result`.
  What it leaves in `result` is what everything after it sees: the inspector's end callbacks, `frame_return_result` and so the caller, and `last_frame_result` for the transaction's frame. An error it returns ends the transaction as an error of `frame_run` would. It leaves the frame stack as it found it.
  A frame that returns a pending action (see The pending action) ran, and its result reaches it.
  A result given without a run does not reach it: `frame_init` returns those (a start the handler refuses, a precompile's answer, an inspector's answer from `frame_start`, `call` or `create`, and an answer of a consumer's own `frame_init`). The consumer's `frame_init` has them first, and an inspector's answer is the inspector's.
  It is on `EvmTr` because both calls reach it there: the inspected run is a provided method of `InspectorEvmTr`, which extends `EvmTr`, and has the EVM but not the handler.
  The plain call is in the handler's loop, not in `Evm::frame_run`, and the inspected call is in the provided `inspect_frame_run`. So an EVM type that wraps revm's `Evm`, delegates `frame_run` to it and keeps the provided `inspect_frame_run` has its own `frame_run_result` called on both paths.
  An override of `run_exec_loop` or of `inspect_frame_run` calls it at the same point itself. An `inspect_frame_run` that delegates to an inner EVM's reaches the inner EVM's `frame_run_result`, not its own, and runs the inner EVM's `frame_end` before the outer type sees the result.
- **Consumer.** mega-evm decides how a frame that crossed its compute limit ends: its out-of-gas becomes a revert carrying the limit's output, with its gas put back. That has to happen once, before an inspector sees the frame end, on both paths. The plain path could do it in `frame_return_result`, but the inspected run calls `frame_end` before that, so a tracer would see the out-of-gas the caller never gets.
- **Removal.** When upstream has an equivalent point between a frame's last action and the inspector's end of it, or when mega-evm rewrites no frame's end.
- **Default.** Does nothing, so every result and every callback is upstream's.
  Where measured (callgrind, the fork's release profile), the default compiles to nothing on the plain path: the costs measured under The pending action are that check's. The inspected path was not measured.

### Start lifecycle

- **API.**
  On `Host`, each with a no-op default: `frame_split(&mut self, split: &FrameSplit)`, `frame_settle(&mut self, settle: &FrameSettle, gas: &mut GasTracker)` and `frame_lifecycle_end(&mut self, end: LifecycleEnd)`.
  `FrameSplit` (`id`, `parent`, `depth`), `FrameSettle` (`id`, `depth`, `status`, `created`, `inspector_answer`, `gas_replaced`, `outcome_rebuilt`, `held_reservoir`), `SettleStatus` (`Ok`, `Revert`, `Halt`) and `LifecycleEnd` (`Settled`, `Discarded`), in `revm_context_interface::host`.
  `FrameSplit` and `FrameSettle` are `#[non_exhaustive]` and have no public constructor: only the record builds them (`FrameStack::open_start`, `FrameStack::close_start`), so a hook is called by hand only with a value the record made, and a field added later breaks no consumer.
  `SettleStatus` and `LifecycleEnd` are exhaustive.
  `revm_handler::lifecycle`: `open_frame_start`, `open_transaction_start`, `Returning` and `close_frame_start`, `close_transaction_start`, `finish_transaction`, `discard_transaction`, `mark_for_end_callbacks` and `check_end_callbacks`.
  On `FrameStack`, the record: `open_start`, `close_start`, `pending_start`, `running_start`, `lifecycle_active`, `lifecycle_settled`, and its two boundaries `settle_lifecycle` and `discard_lifecycle`.
  `StartLifecycleError` (`Exhausted`, `Stale`, `NotOpen`, `NoParent`, `Unsettled`; `#[non_exhaustive]`), `PendingStart` (its fields are private: the identity can be read and not written, and only the three flags can be set), `StartMark` and `OutcomeMark`.
  `GasTracker::start_mark` and `set_start_mark`, and `start_mark` on `CallOutcome` and `CreateOutcome` (a field).
  `LocalContextTr::lifecycle_mirror`, `LifecycleMirror` and `LifecycleEvent` (`Split`, `Settle`, `End`), the debug mirror (below). The types and the method exist in every profile.
- **Contract.** A *start* is one frame's beginning with the identity revm gave it; revm pairs every start it opens with exactly one settlement, last in first out, and the consumer does not pair them.
  *Where revm opens one.*
  - Where it consumes a `NewFrame` action, so the child's start opens against the frame on top of the stack, its caller, at that frame's depth plus one, which is the init's depth:
    the plain `Evm::frame_run` opens it as the frame consumes the action, before `process_next_action` builds the init, and `InspectorEvmTr::inspect_frame_run` right after `process_next_action` returns the init.
    Nothing a consumer can observe happens between the two positions.
  - Where it makes the transaction's first frame: `Handler::execution` and `InspectorHandler::inspect_execution`.
  - In the default `Handler::runtime_oog_result`, for the stand-in result of a transaction whose EIP-2780 runtime phase ran out of gas.

  It opens before `EvmTr::frame_init` on the plain path and before `InspectorEvmTr::inspect_frame_init` on the inspected one, and so before an inspector's `frame_start`, `call` and `create`: an answer made in any of them, an inspector's or a consumer's own, settles on a start that is already open.
  An action an inspector drops or replaces in `step_end` is replaced before revm consumes it, so a start it dropped was never opened, and a clone of the input opens its own.
  A frame init revm answers without running a frame (a precompile, a start it refuses) opens a start and settles it like any other.

  *Where revm settles one.*
  - `Evm::frame_return_result`, after `return_result` applied revm's own settlement, closes the returning child's start, held in the slot of the caller on top of the stack (the caller's own start stays open), and calls `frame_settle` with the caller's tracker.
  - `Handler::last_frame_result`, after `settle_first_frame`, closes the transaction's start and calls `frame_settle` with the transaction's tracker, and after a halt revm consumes the hook's net credit of regular gas and keeps the gas its own settlement left.

  `frame_split` is handed the start's `id`, a per-transaction number revm allocates from 1 with a checked add and never reuses within the transaction; the `parent` identity, the start of the frame that makes this one, `None` for the transaction's frame; the `depth`, read from revm's frame record and never from `interpreter.input`, which an inspector can write; and nothing else: a start's kind, caller, target, value and forward are the frame's own input.
  Whether the consumer's `frame_init` is handed that input after the split is conditional: when initialisation reaches it, it receives the input as an inspector's `frame_start`, `call` or `create` left it; an inspector that answers the start does so before `frame_init` is called, and the stand-in for a runtime out-of-gas transaction has no init at all, so a start can split and settle with no `frame_init` between.
  What a consumer charges or keeps for an instruction's start it does at `frame_starting`, before the split; the transaction's start does not pass through `frame_starting`.
  `frame_settle` is handed the same `id` and `depth` back, and the `status` (`Ok`, `Revert` or `Halt`), `created` (for a creation, whether it took an address), the three flags (below) and `held_reservoir`, the reservoir the caller held before revm settled the child into it.

  *The reservoir.* revm forwards and settles the reservoir as upstream does: the child's input copies the caller's reservoir, or carries the reservoir the frame-start hook named (`FrameStart::reservoir`), and the settlement replaces the caller's reservoir with the one the child returns, once that has unwound the caller's outstanding spill.
  A consumer that keeps part of the caller's reservoir for it takes that part in `frame_starting` and names the rest as the child's, and adds `held_reservoir` back to the caller's reservoir in `frame_settle`.
  The caller then holds what the child returned on top of the part it kept, whatever refills the settlement made, since the unwinding and every refill read the spill and their own amount, never the reservoir.
  The transaction's frame, and the stand-in for a runtime out-of-gas transaction, are handed a copy of the transaction's reservoir, which the transaction's gas keeps until the settlement replaces it, so there `held_reservoir` is that copy: a consumer adds back only what it kept.
  `frame_lifecycle_end` is called once at the end of a transaction's lifecycle: `Settled` at the successful boundary, with every start settled, and `Discarded` when a fatal error ends the transaction and the open starts are dropped unsettled.
  A transaction that ends before any start opened emits neither.

  *The record* is revm's `FrameStack`, which no inspector callback is handed: a per-transaction counter of identities, a slot for the transaction's start and one slot per frame on the stack for the start of the child that frame is making, at most one.
  A child's parent is the start in the slot of the frame under it, and its depth is one more than its caller's.
  Both boundaries reset it.

  *The lifecycle helpers.* A handler or an EVM type that replaces a method revm opens or settles in calls the helper that method calls:

  | It overrides or reimplements | It calls |
  |---|---|
  | `Handler::execution` or `InspectorHandler::inspect_execution` | `open_frame_start` where it makes the first frame's init |
  | `Handler::runtime_oog_result`, building its own stand-in | `open_transaction_start` before it returns |
  | `EvmTr::frame_run` or `InspectorEvmTr::inspect_frame_run` | `open_frame_start` after `process_next_action` returns an init |
  | `EvmTr::frame_return_result` | `lifecycle::Returning::new(&result, caller)` before `return_result` consumes the result, then `lifecycle::close_frame_start` after it |
  | `Handler::last_frame_result` | `close_transaction_start` right after `settle_first_frame`, given the reservoir the caller's gas held before the settlement |
  | `Handler::execution_result` | `finish_transaction`, after the context's recorded error is taken and before the output is built |
  | `Handler::catch_error` | `discard_transaction`, first |

  `finish_transaction` is the successful boundary (`FrameStack::settle_lifecycle`): with every start settled it emits `LifecycleEnd::Settled` and resets the record.
  `discard_transaction` is the fatal boundary (`FrameStack::discard_lifecycle`): it emits `Discarded` only if a lifecycle is under way, resets the record, and is idempotent, so a cleanup that runs twice emits once. It resets the lifecycle only; the frame stack is cleared separately (see Reuse after an uncaught error).
  `close_frame_start` and `close_transaction_start` hand the settlement to `frame_settle` with the caller's tracker and the transaction's, and `close_transaction_start` applies the halt rule after the hook: it consumes only the hook's net regular-gas credit, and a hook that credits nothing or debits leaves the gas as it is.
  A call sits where the mainnet method makes it and passes what that method passes.

  *Fatal errors.* The record refuses these in every build, debug and release alike, as a `ContextError::Custom` carrying the error's `Display`, which fails the transaction and which `catch_error` reports:
  - `Stale`: a start opened where one is still open, so a boundary or a settlement was missed;
  - `NoParent`: a child start under a frame with no start of its own, so a runner drove the transaction's frame without opening its start;
  - `NotOpen`: a settlement found no start open;
  - `Exhausted`: the transaction opened 4,294,967,295 starts; the start does not open;
  - `Unsettled`: `finish_transaction` found a start open. It discards the lifecycle, emits `Discarded` and fails the transaction, so the `catch_error` that reports the error owes no second end.

  The only `debug_assert!`s are on the helpers' own preconditions: the transaction's start opens and closes on an empty frame stack.

  *The marks and the flags.* Before it calls an inspector's end callbacks (`call_end`, `create_end`, `frame_end`) revm writes the start's identity on the result's tracker (`StartMark`, a `u32`, with `GasTracker::set_start_mark`) and a mark of the start on the outcome (`OutcomeMark`, a `u16`, `(id − 1) mod 65535 + 1`, never zero) (`mark_for_end_callbacks`); after them it reads both (`check_end_callbacks`). A consumer has no reason to set a mark.
  `gas_replaced` is set on the start when the tracker's mark is no longer the start's, `outcome_rebuilt` when the outcome's is not, and `inspector_answer` when `frame_start`, `call` or `create` answered the start.
  A constructor (`Gas::new`, `CallOutcome::new`, `CreateOutcome::new`) and a literal that names the mark as `Default::default()` leave it zero, so a value built afresh is always seen; a copy of another start's value is seen unless the two identities are a multiple of 65,535 apart, and a copy made in the previous transaction at the same sequence position is not.
  A tracker an inspector saved before the marking carries none, so restoring it at an end callback sets `gas_replaced` even when it came from the same start.
  The flags say what an inspector did, never what the consumer did: the consumer's own answers are made in `frame_init`, before any end callback, and are never flagged.
  Only the inspected path marks and flags; the plain path leaves all three clear.
  Both marks take no part in equality, hashing, `Debug` or serde: they compare equal always, hash to nothing, are skipped by serde and decode as none, and `GasTracker`, `CallOutcome` and `CreateOutcome` have a hand-written `Debug` that lists the other fields in declaration order, so its output is the derived output without the mark, and `GasTracker`'s encoding has no mark.
  A change to a field of those three types changes that `Debug` too.
  An inspector can forge a mark or call a hook by hand through the `&mut` context: a forged mark changes only the flags, and a hand call desynchronises the consumer's own stack from the record without touching the record or its checks; both are outside the contract.

  *Reuse after an uncaught error.* `Handler::run_without_catch_error` skips the cleanup `catch_error` does.
  A caller that drops its error must do all of it before it reuses the EVM: `discard_transaction`, clearing the local context, discarding the journal (`discard_tx`) and clearing the frame stack.
  A frame stack left as the error left it makes the next transaction's opening a child under a frame whose start was discarded, and it fails with `NoParent`; a record left open fails the next opening with `Stale`.
  Calling `catch_error` does all of it.

  *A hand-driven first frame.* A runner that makes and runs the transaction's first frame itself, or settles one it made, opens the transaction's start first (`open_frame_start` with the frame's init, or `open_transaction_start` for a frame it never initialises), then closes it with `last_frame_result` or `close_transaction_start`, and ends the lifecycle with `execution_result` or `finish_transaction`.
  Without the open, its first child fails with `NoParent` and a direct settlement with `NotOpen`.
  A test that settles a first frame by hand does the same.

  *The debug mirror.* A debug build keeps, in `LocalContext`, a `LifecycleMirror` of the hooks' order that `Context`'s own hooks feed and that panics when an event does not fit: a root split is at depth 0, a child's parent is the identity on top and its depth one more, a settle pops its own identity and depth, and `Settled` finds the mirror empty. It holds the events as `LifecycleEvent`s (`events()`) and the open starts (`open()`).
  The type and `LocalContextTr::lifecycle_mirror` exist in every profile, and a consumer's own ledger can be a `LifecycleMirror`; only `LocalContext`'s field and `Context`'s three hook overrides are debug-only, and a release build holds no mirror and calls none.
  It runs only where a local context returns it from `lifecycle_mirror` (the default returns `None`), and it resets at every root split, so a simulation on a cloned context or an independent EVM nested in an inspector callback checks itself from its own root.
  A consumer's own ledger follows the same rule: it is initialised at the root split, not at construction or at clone time, and dropped at either end.
  A context type that wraps `Context` and implements `Host` itself forwards the three hooks to the inner context, or the mirror never runs; the record and its checks run either way.
- **Consumer.** A consumer that keeps per-start accounting keeps an entry for each start on a stack of its own, drives it only by `frame_split` and `frame_settle`, and ends it at `frame_lifecycle_end`; it does not pair a start with its settlement itself.
  A consumer that pairs them from the frames it sees breaks when an inspector replaces an interpreter, rewrites an input's depth or drops an action.
  The flags are what such a consumer needs of an inspector's answers, rewrites and rebuilds to settle them as it would without the inspector.
  The lifecycle carries no MegaETH figure; what a start's entry holds is the consumer's.
- **Removal.** When upstream exposes an equivalent pairing of a frame's start with its settlement, or when mega-evm keeps no accounting per start.
- **Default.** The hooks do nothing and are inlined away, the flags are set on the inspected path and read by no one, and every result and every callback is upstream's.
  What a consumer that sets nothing pays is revm's own record and the marks, in bytes: the record adds 40 bytes to `FrameStack`, which grows from 40 to 80 bytes per EVM, and a slot of 8 bytes (`PendingStart`) on the heap and reused: the slots are sized with the frame stack's capacity (for zero-sized frames, one per frame that made a child), grown when a child first opens past them and never shrunk: a boundary neither resizes nor frees the slots, a successful end does not scan them, and discarding open children clears the retained slots, at a cost proportional to their retained length.
  The marks sit in the padding of the values they ride on, so `GasTracker` stays 88 bytes, `Gas` 104, `InterpreterResult` 144, `CallOutcome` and `FrameResult` 208 and `CreateOutcome` 192 (64-bit targets), and a release build's `LocalContext` stays 32 bytes; a debug build's is larger for the mirror.
  In instructions, a start costs a checked increment, a slot write and a parent read, a settlement a slot take, a transaction one reset at each boundary (and, after a discard with open children, one clear of the retained slots), and an inspected frame's end two stores and two compares; the cost per frame, in instructions, is measured separately and not given here.
  Where measured, the three hooks leave no symbol in an unstripped optimised build; what the loop gains is a call to `FrameStack::open_start`.
  `Host`, `LocalContextTr` and `FrameStack` gain only default methods and private fields, and `revm_handler` a module, so they break no code written for upstream's.
  Three exceptions to that compatibility are carried, each an `api:exception` item.
  A pull request that introduces such an exception lists it and checks the consumers against it.
  What a consumer does for each:
  - The outcomes' public `start_mark` field (`CallOutcome` and `CreateOutcome`): a consumer that builds an outcome by literal names it as `start_mark: Default::default()`, or builds it with the constructors. It extends the exception the outcome literals already carry for the state-gas refund's address (see State-gas price).
  - `LocalContext` gains a private field in a debug build, so a debug build cannot build it with a struct literal; a consumer builds it with `LocalContext::default()` or `new`. A release build's is unchanged.
  - A behaviour change no compiler finds: a runner that drives the transaction's frame, or settles it, without `Handler::execution` and `Handler::last_frame_result` must open and close the transaction's start (see A hand-driven first frame); otherwise the record refuses it with `NoParent` or `NotOpen`.

### The original account info by reference

- **API.** `revm_state::Account::original_info_ref(&self) -> &AccountInfo`.
- **Contract.** It returns what `Account::original_info` returns, by reference, cloning nothing: the info the account holds as its original, or the default info for an account whose original was never set.
- **Consumer.** mega-evm tells whether a transaction changed an account by comparing the account's info with its original, for every frame start, value transfer and destruction it counts. `original_info` returns a copy, and copying the info's code moves the code's reference count up and down; the original is a private field, so only the crate can lend it.
- **Removal.** When upstream has an equivalent accessor, or when mega-evm stops comparing an account with its original.
- **Default.** Nothing in this workspace calls it, so every result is upstream's. The default info it lends an account without an original is built on the first such read and kept for the process.

## Baseline behaviour the fork changes

Picked upstream fixes change revm 40.0.3's results where 40.0.3 was wrong; that is why they are taken (see Tracking upstream), and each pick's pull request says what it changes.
Once a consumer spec that runs on the fork is sealed, a pick that changes a result it can reach lands behind a switch that keeps the sealed result, or is not taken (see Tracking upstream).
A pick that changes what the journal records lands with a switch back to the base line (see revm 40.0.3's journal semantics).
One pick changes what the top-level entry points do with the journal, and has no switch.
The pending-action check changes what an inspector that sets an action before a frame runs sees and gets, and has no switch either:

- **The journal is finalized on an error.** `ExecuteCommitEvm::transact_commit`, `ExecuteEvm::replay`, `InspectEvm::inspect_tx` and `inspect`, `InspectCommitEvm::inspect_tx_commit` and `inspect_commit`, and the four `inspect_system_call` variants clear the journal when execution returns an error.
  revm 40.0.3 left it holding the failed transaction's warm set and touched accounts, which leaked into the next transaction run on the same EVM.
  Upstream #3780, carried verbatim with the EIP-8037 / EIP-2780 gas core.
  There is no switch: a consumer that must keep the journal after an error uses the entry points that do not finalize (`transact_one`, `inspect_one_tx`, `system_call_one`) and finalizes itself.
- **An action an inspector sets in `initialize_interp` is returned without a `step`.** The inspected run returns an action pending when it starts before its first `Inspector::step` (see The pending action).
  Only an inspector that sets an action before a frame runs, by halting the frame or setting its next action in `initialize_interp`, is affected: it loses that run's first `step`, and with it whatever that `step` would have done.
  revm 40.0.3 called `step` once first, at the frame's first instruction, and then returned the action as that `step` left it, without running the instruction.
  An inspector whose `step` only observes sees one `step` fewer and gets the same result, since no instruction runs on either.
  An inspector whose `step` edits the pending action (`LoopControl::action`), or changes the context or the state, gets a different result: the frame returns the action `initialize_interp` set, and the `step`'s changes are not made.

## Tracking upstream

The fork keeps its crate versions at the revm version the MegaETH node pins, today `40.0.3`; a newer upstream tag would bump every crate's major, so the fork takes single upstream commits until the node moves.

Taken: changes to the EIP-8037 / EIP-2780 gas core whatever their prefix; fixes that change execution results (gas, state, logs, halt reasons, precompile output, bytecode analysis, serialized formats); correctness and safety fixes relevant to the consumers, including undefined behaviour, missing out-of-gas checks, panics, and dependency or build failures; performance improvements to execution, precompiles and the database; each with the test-fixture bump that comes with it.
A change that moves an execution result is taken this way only until the consumer spec that runs on the fork is sealed; after that it follows the sealed-spec rule below, whatever its kind.
Not taken: tooling-only changes, feature-gated additions the consumers do not use, and change/revert pairs, after checking that no taken commit depends on them.
Picks are batched by crate or subsystem, one pull request per batch, so one reviewer can read a whole batch.
The fork has no benchmark gate: a performance pick must leave execution results unchanged under the crate tests and the EEST run, and cite upstream's benchmark numbers in the pull request body.
A pick that touches a precompile is tested under mutually exclusive feature sets, because `--all-features` selects only the preferred backend in each `cfg_if!` block: `--no-default-features --features std` (every fallback: the arkworks BLS12-381 and KZG paths, `aurora-engine-modexp`, `k256`, `p256`), the default set, `--all-features`, and the single feature that selects the backend the pick changes (`std,blst`, `std,c-kzg`, `std,gmp`, `std,secp256k1` or `std,p256-aws-lc-rs`) on its own.
A pick that breaks the public API needs `api:exception`, the deviating items listed in the pull request body, and a check of every consumer that uses them.
An upstream change that upstream later reverted but the fork still needs lands as a fork-owned opt-in switch (`mega:hook`) whose default keeps upstream's behaviour.
A picked upstream fix that changes journal semantics lands with an opt-in switch back to the base line's behaviour when a consumer reproduces the base line exactly; the switch defaults to the picked behaviour, the consumer that needs the base line sets it, and nothing else reads it. `CfgEnv::legacy_journal_semantics` (the cold-slot re-baseline and the `CodeChange` revert of revm 40.0.3) is the first.

**Sealed specs.** Once a consumer spec that runs on the fork is sealed, the fork must not move that spec's results.
A pick that changes an execution result the sealed spec can reach then lands behind a switch, or is not taken.
The switch follows the `legacy_journal_semantics` pattern: it keeps the result the spec was sealed with, its default is the picked behaviour, which is upstream's, and the consumer turns it on for each sealed spec, so a later spec can take the fix by leaving it off.
A pick that cannot be put behind such a switch is not taken while a sealed spec runs on the fork, and its sweep records why.
A pick that changes only results no sealed spec reaches, such as a fix to a feature the sealed specs leave off, lands without a switch, and its pull request says why no sealed spec reaches it.
A pick lands without a switch only with both of these in its pull request: a code-based argument that names the behaviour the pick changes and shows that no sealed spec reaches it, and targeted regression tests for that behaviour.
The consumer's outcome check on each sealed spec, run against the pick, is supporting evidence and is recorded too; a fixture corpus shows only the cases it reaches, so passing it does not define the pick as safe.

Every pick is applied verbatim: `git show -U0 <commit> | git patch-id --verbatim` gives the same id for the pick and its upstream commit unless the pick's message records a conflict resolution; the pull request body, which becomes the squash message, lists each pick's `(cherry picked from commit <sha>)` line and its conflicts.
A pick that lands behind a switch keeps the verbatim pick as its own branch commit, followed by the switch in a commit of its own, and the squash message records the switch as the pick's conflict resolution.

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
| `mega:hook` | A hook, data field or switch, or an item of rule 2's MegaETH accounting |
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
