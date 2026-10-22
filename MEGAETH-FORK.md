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

This section lists every hook and switch the fork carries.
Each entry names the API a consumer uses and what the fork guarantees about it.
A hook's default keeps upstream's behaviour: a consumer that does not use it sees the same results.
Each entry's **Consumer** says who needs it and why it cannot be done outside the fork, its **Removal** the condition under which the fork drops it, and its **Default** what it costs a consumer that does not use it.

A field a hook adds to a serialized type carries `serde(default)`, so in a format that carries field names, such as JSON, an encoding made before the field existed decodes with the default.
A format that encodes a struct by position, such as bincode, cannot read such an encoding, as with upstream's own field additions: it reads the old fields in their places and then looks for the new ones, so a value persisted in such a format must not cross an upgrade of the fork that adds a field.

A gas schedule (`GasParams`) is a table of 256 entries, not named fields, so `serde(default)` cannot reach an entry: a table built raw (`GasParams::new`) or decoded from an encoding made before an entry existed holds zero there.
Every entry the fork adds is therefore zero by default, and zero is upstream's behaviour: such a table behaves as the default schedule does.
An id the fork once used for an entry with another default is left unused rather than given a new meaning, so a table encoded then does not read as a setting.

### State-gas price

- **API.** `Host::state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64>` and `Host::state_gas_charge(&mut self, charge: StateGasCharge) -> Option<u64>`.
  The value types `StateGasSite` (`account`, `slot`) and `StateGasCharge` (`one`, `units`, `total`) in `revm_context_interface::cfg`.
  For the refund: `charged_state_gas_address` on `CreateInputs`, `CallOutcome` and `CreateOutcome`, and `FrameResult::refundable_state_gas_charge`.
  `pre_execution::Eip7702AuthFacts`, the authorization list lifted out of the transaction so the authorization phase can hand the whole context to the hook.
- **Contract.** Every EIP-8037 state-gas charge and every refund of one is priced through `state_gas_price`, so a charge and the refund that undoes it agree by construction.
  `site` names the account the charge lands on, and the storage slot for a slot-scoped charge; a creation is priced at the address it deploys at, the first frame's included.
  `None` reports a failed lookup: the implementation records the cause in the context error, and the charge site turns it into a fatal external error.
- **Consumer.** mega-evm prices state gas by the storage bucket of the account or slot a charge lands on. Only the charge site knows the site, and the price must be the same at the charge and at its refund.
- **Removal.** When upstream prices state gas through an equivalent hook, or when mega-evm prices every state-gas charge from the flat schedule.
- **Default.** Reads the flat schedule (`gas_params().get(id)`) and never fails, so every charge and refund is upstream's.
  The refund carries one address more in the frame inputs and outcomes, and under EIP-2780 an EIP-7702 transaction's authorization list is buffered before the authorization phase, one 88-byte entry per authorization that can apply.

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
  It takes two words of the tracker, a number and a flag, and the flag's word also holds the pools' crossings (see Resource pools); every copy of the tracker, and of every `Gas` and frame result that holds one, carries both words.
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

- **API.** `ContextTr::admit_code_deposit(&mut self, deposit: &CodeDeposit<'_>, gas: &mut GasTracker) -> Result<(), Bytes>`, whose default admits and leaves `gas` alone.
  `CodeDeposit` carries `address`, `code`, `gas_before` and `gas_after`; `regular_gas`, `state_gas` and `history_gas` read the deposit's charges off the two trackers.
  It is `#[non_exhaustive]` and built with `CodeDeposit::new`, so a later field is not a breaking change.
- **Contract.** `return_create` calls it once for every creation it deploys, on the plain and the inspected path: after the code passed every check and every deposit charge is recorded, before the journal checkpoint is committed.
  A creation `return_create` fails itself, out of gas on a deposit charge included, is never offered.
  Before Homestead a deposit the frame cannot pay does not fail: `return_create` deploys empty code instead, and offers that deposit with empty code and no charge.
  If the failed charge was a crossing of the frame's withheld gas, `gas_after` carries its crossing record and `gas_before` does not, so an admitted deposit keeps the record and a refusal discards it.
  `gas` is the creating frame's tracker, which `gas_after` reads a copy of: the hook may debit and credit it, for instance draw the deployed code from one of the frame's pools (see Resource pools), and a charge it makes that fails records as any failed charge does.
  `Ok(())` admits, and the frame returns with `gas` as the hook left it.
  `Err(output)` refuses: the checkpoint is reverted, the frame's gas is put back to `gas_before`, which undoes whatever the hook did to `gas`, a crossing its failed draw recorded included, and the creation ends as a `Revert` with `output`, settling as a creation whose init code reverted.
  A hook that fails fatally records the cause in the context error and refuses; the transaction returns the error.
  The deposit's charges, and with them its state-gas price lookup through `Host::state_gas_price`, are made before it is offered, so that lookup is made whether or not the deposit is admitted; a refusal does not undo what the lookup recorded.
- **Reach.** The hook is on the context `return_create` is called with, so a consumer that wraps `EthFrame`, runs `process_next_action` from its own `frame_run`, or customises `Handler` reaches it by implementing `ContextTr` for its context.
  A context that wraps another and implements `ContextTr` by hand must forward the method, or it gets the admitting default.
- **Consumer.** mega-evm refuses a deposit its own rules reject, after revm's checks and charges and before the creation commits, and holds the deployed code to the creating frame's byte budget there. That point is inside `return_create`, between its last charge and the checkpoint commit, and no other hook is lent the frame's gas between the two.
- **Removal.** When upstream has an equivalent admission point, or when mega-evm stops refusing deposits.
- **Default.** `return_create` compiles to the same code, on its own and inlined into the handler on the plain and the inspected path.
  Inlined into a consumer's own function, the admitting default may shift its block layout; where measured, that cost one unconditional branch on one out-of-gas path and changed no success path.
  Where measured (callgrind, the fork's release profile), lending `gas` costs nothing: a top-level creation runs the same instruction count as without it, and a nested `CREATE` 0.06 fewer.
  The `gas` parameter is an exception to rule 1 (`api:exception`): an override written for the signature without it does not compile until it takes `gas: &mut GasTracker`, and an override that leaves `gas` alone behaves as it did without it.
  A context that forwards the method to another forwards `gas` with it.
  Nothing built against upstream revm overrides the hook, since the hook is the fork's.

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
  `FrameStart` (`#[non_exhaustive]`, built with `FrameStart::new`: `kind`, `caller`, `target`, `value`, `forward`, and `charge`, which `new` leaves empty) and `FrameStartKind` (`Call`, `CallCode`, `DelegateCall`, `StaticCall`, `Create`, `Create2`), in `revm_context_interface::host`.
  `FrameStartKind` is exhaustive on purpose, as `CompletedWrite` is: mega-evm matches it to charge each start, so a new kind is a compile error in every consumer that matches it rather than a start that passes a wildcard arm uncharged, and adding one is an `api:exception` change.
- **Contract.** `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2` call it after every charge they make and after the forward is taken from the caller's gas, before they build the frame's input, on the plain and the inspected path.
  `forward` is the regular gas taken: the frame's gas limit without a value call's stipend. `value` is the value transferred, zero for `DELEGATECALL` and `STATICCALL`.
  `target` is the account the frame runs as: the account called for `CALL` and `STATICCALL`, the calling frame's own for `CALLCODE` and `DELEGATECALL`, and for `CREATE` and `CREATE2` the address the creation would take.
  Whether a creation takes it is the outcome's to say (`CreateOutcome::address`): a creation that lands on an account that already has code or a nonce, one the handler refuses after the hook (an inspector changed the caller's balance), and one an inspector's `create` or a consumer's `frame_init` answers without a frame take none, though their start named the address. A consumer that counts the created account from `target` settles that count on the outcome.
  A creation's address is computed once, into the creation's input, and the handler builds the frame at it without computing it again. With EIP-8037 on, the instruction's EIP-8037 branch computes it. With EIP-8037 off, the instruction computes it after the same checks, reading the caller's balance and nonce from the journal.
  It is `None` only for a creation the handler fails before it takes an address: an endowment above the caller's balance, a caller's nonce at its maximum, or a creating frame at the depth limit. Such a creation reaches the hook only with EIP-8037 off; with it on, the instruction ends the creation before the hook.
  The address is fixed when the hook sees it. Something that changes the caller's nonce between the instruction and the frame does not move it, as on upstream's EIP-8037 path. An inspector that changes the creation's caller, scheme or init code (`CreateInputs::set_call`, `set_scheme`, `set_init_code`) clears it, and the handler computes the new one.
  `gas` is the caller's tracker. The hook may debit and credit it, and the instruction uses it as the hook left it: nothing is charged after the hook, and the frame's input takes its share of the reservoir from it, so a frame the handler runs and one it answers without running (a precompile) both start from a share of the reservoir the hook left (see The reservoir share). A charge the hook makes that fails records as any failed charge does, whatever the answer.
  `start` is the hook's to declare the start charge on (`charge`, see The start charge); the instruction reads nothing else back from it.
  After the hook the instruction copies that charge onto the frame's input and takes the child's shares of the reservoir and of each pool off `gas` for it (`GasTracker::forward_to_child`, see The reservoir share and Resource pools); the shares are of the reservoir and the pools as the hook left them.
  `true` hands the start to the handler, which may still refuse it (call depth, balance) or answer it without a frame; what the hook did to `gas` stands either way. `false` fails the instruction out of gas and starts no frame: the forward stays taken unless the hook gave it back, and the halt records the regular gas the hook left unless a failed charge already did.
  A creation the instruction ends before its forward (EIP-8037's checks of the endowment, the nonce and the depth) is not offered.
- **Consumer.** mega-evm charges a start's write records after the forward and before the frame inherits the caller's reservoir. Without the hook it wraps the call and creation instructions, rewrites the reservoir revm already copied into the input, and takes the input back when the charge fails.
  It counts the created account's record only when the transaction has not changed that account already (funded it, say), so it needs the address at the start. A `CREATE2`'s address depends on the init-code hash, which only the instruction has.
- **Removal.** When upstream exposes an equivalent point, or when mega-evm stops charging frame starts.
- **Default.** Returns `true` and touches nothing.
  Where measured (callgrind, the fork's release profile), a call runs one instruction more than without the hook: the store of the forward its helper returns, about 0.05% of a call to a contract that stops.
  For `CREATE` and `CREATE2` the hook's only cost is the start's address; the byte pool's forward is counted under The byte pool.
  With EIP-8037 off, the default spec's path, the address costs a warm journal read of the caller and the three checks: 149 instructions per `CREATE` and 147 per `CREATE2` from a contract (+0.95% and +0.73%). The read is a `Host::load_account_info_skip_cold_load` call that upstream's instruction does not make, so a `Host` that counts or refuses loads sees one more per creation.
  The instruction computes the address and a `CREATE2`'s init-code hash, and the handler reuses them, so a creation the handler builds or fails computes the same hashes as upstream's.
  A start that never reaches the handler's frame building computes them for the hook alone: one hash for a `CREATE`, two for a `CREATE2`. That is a start the hook refuses, and a creation answered without a frame, by an inspector's `create` or by a consumer's own `frame_init`. Upstream's EIP-8037 path computes them before the hook in the same way.
  With EIP-8037 on, the start reads the address the EIP-8037 branch already computed from the input; that path was not measured.
  `CreateInputs`' derived `PartialEq` compares the cached address and hash, which the input of a creation the instruction starts holds with EIP-8037 on and off, so it compares unequal to an otherwise equal input that has computed nothing, and to its own serde round trip, which skips the cache.
  A consumer tells a creation from a call by `kind`, not by `target`: `target` is `None` only for the creations the Contract names, and for every other start an address, which for a creation is the one it would take.

### Resource pools

- **API.** `PoolId` (`new(index)`, `index`, `COUNT` = 3, `ALL`) and `PoolAmounts` (an amount per pool: `ZERO`, `UNLIMITED`, `get`, `set`, `with`, `is_zero`), in `revm_context_interface::cfg::gas`, re-exported from `revm_interpreter`.
  On `GasTracker`, forwarded on `Gas`: `pool_remaining`, `set_pool_remaining`, `pool_spent`, `set_pool_spent`, `record_pool`, `refill_pool`, `pool_crossing`, `clear_pool_crossing`, `set_pool_crossing`, each taking the `PoolId`.
  On `GasTracker` alone, the fork's own plumbing: `pools_remaining`, `set_pools_remaining`, `add_pool_spent`, `forward_pool`, `absorb_returned_pool`, `absorb_returned_pools`, `adopt_pool_share`, `adopt_pool_shares`, and `forward_to_child` and `forward_all_to_child` with their `ChildShares` (see The reservoir share).
  `GasId::pool_kept_percent(pool)` (ids 248, 247 and 246) and `GasParams::pool_forward_share(pool, remaining)`.
  `pool_shares` on `CallInputs` (a field) and `CreateInputs` (`pool_shares`, `set_pool_shares`).
- **Contract.** A tracker carries `PoolId::COUNT` pools, budgets beside its gas whose meaning the consumer gives (bytes a frame may append, records it may write, entries it may create); the fork knows them by index only.
  For each pool, `pool_remaining` is what the frame may still draw and `pool_spent` its net draw, signed as `history_gas_spent` is.
  `record_pool(pool, n)` draws; a draw the pool cannot pay takes nothing, returns `false` and sets the pool's crossing (`pool_crossing(pool)` is `true`), which survives `spend_all`, starts clear in every child, is not taken over by a caller, and stays until cleared; a consumer marks a result it produced itself with `set_pool_crossing`.
  A crossing is a bit per pool: it says that a draw failed, not what the draw asked for, which the hook that drew knows when it fails.
  No halt touches a pool, so a halted frame's result still reads each pool as the failed draw found it, until its caller settles it.
  `refill_pool(pool, n)` gives back and lowers the net, below zero when the draw was a caller's.
  Amounts are bounded as the tracker bounds gas amounts: every draw and refill is at most `i64::MAX`, and each net with its children's nets merged in stays within `±i64::MAX`, so it can be negated; a debug build asserts it where a net moves, and past it a release build saturates the net, so a rollback no longer restores the pool exactly.
  Each pool is a figure of its own: nothing the fork does to one moves another, and a consumer that draws several for one write draws them one by one, in the order its rules give.
  A consumer draws deployed code where the deposit is admitted: `ContextTr::admit_code_deposit` is lent the creating frame's tracker (see Code-deposit admission).
  The transaction's frame gets every whole pool of the transaction's tracker, set by a consumer at `tx_gas` or in pre-execution (`forward_all_to_child`, from `create_init_frame` and from the stand-in `Handler::runtime_oog_result` builds when the EIP-2780 runtime phase runs out of gas).
  At every `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2`, after `frame_starting` has answered, the instruction takes `pool_forward_share(pool, …)` of each of the caller's pools out of it and puts the shares on the frame's input as `pool_shares`: the child's pools when it starts, on the frame the handler runs and on an answer it gives without one (a precompile, a refused start).
  `pool_forward_share` is `100 − pool_kept_percent` per cent of what the caller has left of the pool, rounded down, and the caller keeps the rest: zero kept forwards the whole pool, a hundred or more none.
  A forward is not a draw.
  The settlement is in `handle_reservoir_remaining_gas` (`absorb_returned_pools`) and follows the reservoir, not regular gas: a child that reverts or halts has its draws on every pool undone by `rollback_state_gas`, which gives each pool's net back to it, so what the caller absorbs of each pool is the child's remaining pool on success and the whole share on revert *and on halt*, since a halted frame draws nothing; the caller kept what it did not forward.
  The child's nets merge into the caller's on success only, as the history net does, so a caller's rollback undoes its children's draws too.
  `rollback_state_gas` rolls every pool back with state and history gas, so a consumer that rolls a frame back by hand rolls the pools back with it.
  The first frame's return settles into the transaction's tracker the same way (`last_frame_result`, through `execution::settle_first_frame`, see The start charge), whose `pool_spent` of each pool is then the transaction's net draw.
  A precompile's answer adopts the shares as the pools it started with (`adopt_pool_shares`): each pool is the share less what its provider drew on it, so its pools and nets add up to the shares as a frame's do; a provider that draws on its answer starts the answer's pools at the input's `pool_shares` first, so a draw a share cannot pay fails.
  A provider that drew more than a share anyway, on pools it did not start at the shares, fails closed on that pool: the adoption empties the pool, caps its net at the share and sets its crossing, so the caller gets back no more than it forwarded whether the answer succeeds or fails, and the consumer classifies the crossing as it classifies a refused draw. The answer's result is the provider's, so a successful answer can carry a crossing.
  An outcome an inspector builds to answer a start carries whatever pools its gas was built with, unlimited ones with `CallOutcome::new` and `CreateOutcome::new`, and the settlement absorbs them as they are.
  A consumer that answers a start itself, without a frame or a provider, sets the answer's pools to the shares it was given (`set_pools_remaining(inputs.pool_shares)`), and a consumer that builds a child's input itself takes the child's shares with `forward_to_child`.
  The pools are not reported on `ResultGas`.
- **Consumer.** mega-evm keeps per-frame budgets of what a transaction adds to the chain: the bytes it appends to the state and the history, the write records it makes and the state entries it creates, each forwarded to a child at 98/100 of what its caller has left, each given back by a failed frame and handed to its caller by a successful one, and each held to a per-transaction and a per-block limit. That is what the tracker does for the reservoir, and only the tracker's forward and settlement can carry figures that move with a frame's gas.
- **Removal.** When upstream carries equivalent per-frame budgets on its gas, or when mega-evm stops limiting these per frame.
- **Default.** Every tracker starts with every pool unlimited (`u64::MAX`), every schedule keeps none of a pool and so forwards every whole pool, and nothing in the workspace draws one, so every figure of the pools is the unlimited pool going out and coming back and every result is upstream's.
  A table without the entries, built raw or decoded from an encoding that predates them, keeps none too (see Hooks).
  Three pools, because mega-evm limits three figures per frame: data size, write records and state growth. A fourth would carry one more figure the engine keeps beside the pools today (the transfer-log bytes it subtracts from the history bytes), but the engine's per-start entry stays for the take-back credit and the creator-record overdraft in any case, and the pool would cost every consumer 16 bytes more on every frame result. A later pool is a size change, not an API change: it moves `PoolId::COUNT` and `PoolId::ALL`, gives the pool's kept percentage the next id below the pools' (245) and a name, and changes the tests that count them.
  The tracker is 144 bytes (88 without the fork's figures, 112 with one pool): two words per pool, the pool and its net, a word for the allowance (see The history allowance), and the crossings, a bit per pool, in the word the failed-charge record's flag takes (see The failed-charge record). In a format that carries field names, encodings that predate the fields decode with unlimited pools, no crossing and zero for the rest, and a positional format cannot read them (see Hooks).
  `Gas` is 160 bytes (104 at the base without the fork's figures), `InterpreterResult` 200 (144), `CallOutcome` and `FrameResult` 296 (208), `CreateOutcome` 280 (192), the outcomes with the start charge's four words (see The start charge).
  Where measured (callgrind, aarch64, the fork's release profile), against the single byte pool before these pools: a transfer costs 215 instructions more (+3.9%), each `CALL` about 258 more (+10.9% on a transaction of 64 calls), a top-level creation 273 more (+1.5%), a nested `CREATE` 331 more (+2.1%) and a `SELFDESTRUCT` transaction 285 more (+3.5%).
  Nearly all of it is the size: the same code with one pool, at the earlier sizes, cost 2, 8, 7, 10 and 1 instructions on the same workloads. At 296 bytes the frame result passes what the compiler copies inline, and the moves of the frame results through the handler's `Result` and `ItemOrResult` wrappers (about seven per `CALL`) become library `memcpy` calls.
  Against the base without any of the fork's figures, a transfer costs 336 more (+6.2%) and each `CALL` about 398 more (+17.9% on a transaction of 64 calls).
  A precompile's answer runs 11 instructions more than with an adoption that does not check each pool's net against its share (see Contract); measured against the head before the check, a transfer runs 1 instruction less, each `CALL` 1.9 less, a top-level creation 3 less and a nested `CREATE` 7.75 less, and a `SELFDESTRUCT` transaction 4 more, from code layout, not from the check.

### The reservoir share

- **API.** `GasId::reservoir_kept_percent` (id 249) and `GasParams::reservoir_forward_share(reservoir)`.
  On `GasTracker`: `forward_reservoir(share)`, and `forward_to_child(&GasParams)` and `forward_all_to_child()`, which take the reservoir's share and every pool's share at once and return them as `ChildShares { reservoir, pools }` (`#[non_exhaustive]`).
- **Contract.** At every `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2`, after `frame_starting` has answered, the instruction takes `reservoir_forward_share` of the caller's EIP-8037 reservoir out of the caller's tracker and puts it on the frame's input as `reservoir`, the child's reservoir when it starts, on a frame and on an answer alike.
  `reservoir_forward_share` is the reservoir less the caller's part, `reservoir_kept_percent` per cent of it rounded down, taken away as `call_stipend_reduction` takes regular gas: at 2 kept the child receives `reservoir − reservoir × 2 / 100`, so a reservoir of 99 forwards 98, where a pool keeping 2 forwards 97 (its share is rounded down). Zero kept forwards the whole reservoir, as EIP-8037 does; a hundred or more forwards none.
  The two roundings are the fork's, not the schedule's: the reservoir's share takes the caller's part away as regular gas's forward does, and a pool's share is rounded down. A chain that needs the floor for the reservoir, or the subtraction for a pool, has no entry for it.
  The caller keeps the rest while the child runs; the share is all the child has, so a child whose state and history charges pass it spills onto its own regular gas, as a frame with an empty reservoir does, and one that cannot pay that fails out of gas.
  On the child's return, `GasTracker::absorb_returned_reservoir` adds the reservoir the child left to the part the caller kept, where upstream's replaced the caller's reservoir with the child's; as before, the returned reservoir first unwinds the caller's outstanding spill, last in first out, and the part the caller kept stays in its reservoir.
  A child that fails rolls its charges back and returns its whole share; one that succeeds returns what it left, its own refills (EIP-8037 restorations) included, which land in its reservoir as before.
  A caller that keeps part of its reservoir can therefore hold it beside an outstanding spill: a child that succeeded after spending past its share spilled onto its regular gas, and the spill merges into the caller's. A whole forward cannot produce this, since the caller then keeps none.
  A refill in such a caller (an EIP-8037 restoration, `refill_history`, the refund of an upfront or a start charge) keeps EIP-8037's order: regular gas up to the outstanding spill first, the reservoir after, whichever of the two paid the charge the refill undoes. So a charge the caller paid from its kept reservoir comes back as regular gas while a child's spill is outstanding.
  For example, a caller with a reservoir of 1,000 that keeps 2 per cent keeps 20 and forwards 980; a child that charges 1,000 spills 20; the caller then charges 10 from its reservoir (reservoir 10, spill 20) and refills 10, which leaves its reservoir at 10, its spill at 10 and its regular gas 10 higher.
  The sum of the two is conserved, a rollback still restores the reservoir the frame started with, and the reimbursement and `gas_used` read the sum, so no final figure moves; `GAS` in the caller reads the credited regular gas.
  The transaction's frame takes the whole reservoir of the transaction's tracker (`forward_all_to_child`, from `create_init_frame`), and settles back into it the same way, so a system call's frame does too; so does the stand-in result `Handler::runtime_oog_result` builds for a transaction whose EIP-2780 runtime phase ran out of gas, which carries the whole reservoir and every pool and settles them back once.
  A precompile's answer carries the share the input gave it, as the provider builds the answer's gas from the input's reservoir (the standard providers do, and a provider must); the fork does not check an answer's reservoir, state-gas net or spill against what the input forwarded, and takes the provider's answer as the provider reports it, as upstream does, so only the pools are adopted fail-closed; a start the handler refuses (call depth, balance) and a creation it fails early carry it too.
  An outcome an inspector builds to answer a start carries whatever reservoir its gas was built with, none with `Gas::new`, and the settlement adds it to what the caller kept: with the whole reservoir forwarded the caller is left with none, as with upstream's settlement.
  **The settlement adds.** `handle_reservoir_remaining_gas` (with the `GasTracker::absorb_returned_reservoir` and `absorb_returned_pools` it calls) and `execution::settle_first_frame` add what a frame returns to what its caller holds, so the caller must no longer hold what it handed the frame.
  Code that hands a frame the caller's reservoir or a pool without taking it from the caller compiles, and the caller ends with it twice. The shapes:
  - a child's input built by hand from the caller's `reservoir()` or `pools_remaining()`, while the caller keeps them;
  - the transaction's frame, or a stand-in result for it, built from the transaction's gas without taking from it, in an override of `Handler::first_frame_input` or `Handler::runtime_oog_result` or in a runner of its own;
  - a runner that settles the transaction's frame into gas it seeds from the frame's own figures (`GasTracker::new(limit, frame.remaining(), frame.reservoir())`) and then calls `last_frame_result` or `settle_first_frame`.

  A consumer takes what it hands over: `forward_to_child` takes the schedule's shares, as the instructions do; `forward_all_to_child` takes everything, as the transaction's frame does; `forward_reservoir` and `forward_pool` take a share the consumer computed.
  A consumer that lowers a child's share after the forward and gives the excess back to the caller keeps the sum, and is right; so is code that settles with arithmetic of its own that replaces the caller's reservoir and calls none of these functions.
  A debug build asserts on entry to `settle_first_frame` that the transaction's gas holds no reservoir and no limited pool; an unlimited pool is exempt, since what comes back saturates at it. A frame below the transaction's has no such check: its caller may hold a kept part, or an excess given back, beside the share, and only the code that built the input knows which.
- **Consumer.** mega-evm keeps the caller's part of the reservoir from a callee that could exhaust it: a child receives 98/100 of what its caller has left, as with every other resource a callee could exhaust. Only the instruction knows the reservoir as the frame-start hook left it before the input is built, and only the settlement can add the child's reservoir to what the caller kept.
- **Removal.** When upstream forwards a share of the reservoir through an equivalent setting, or when mega-evm forwards the whole reservoir.
- **Default.** Every schedule keeps none of the reservoir and so forwards it whole, the child's return gives the caller what the child left, and every result is upstream's: EEST, the default-path figures pinned to upstream and the handler suites pass unchanged.
  A table without the entry, built raw or decoded from an encoding that predates it, keeps none too (see Hooks).
  What a frame's tracker reads while its child runs differs: its reservoir is the part it kept, zero by default, where upstream's left the forwarded reservoir in it until the child returned; an inspector that reads the caller's gas at the `CALL` or `CREATE` instruction's `step_end` sees it.
  Where measured with the pools' figures (above), the share and the generalised forward cost 8 instructions per `CALL` with one pool.
  `handle_reservoir_remaining_gas` (upstream's, public) and `GasTracker::absorb_returned_reservoir` (upstream's, picked) change meaning: they add the child's reservoir to the caller's. A consumer that calls them, or builds a child's input, without taking the share from the caller does not fail to compile; it counts the reservoir twice. The pull request that makes the change lists it with the `api:exception` items.
  A consumer that builds a child's input by hand with the caller's reservoir, without taking the child's shares with `forward_to_child`, counts that reservoir twice, and must take them. A first-frame settlement written outside the handler does the same. A consumer that runs before the reservoir exists, or settles its first frame without `handle_reservoir_remaining_gas` or `GasTracker::absorb_returned_reservoir`, calls neither function and is unaffected.
  Upstream's own builders of a first-frame result take the reservoir and the pools out of the transaction's tracker: `create_init_frame` and `Handler::runtime_oog_result`, whose default body changes so, which an override that builds its own stand-in must do too (see The settlement adds).
  The entries are kept parts, not forwarded ones, so that zero, which a table without them holds, forwards everything. Ids 254 and 250 to 252 are left unused. A consumer names the entries by function, and a chain that forwards 98 per cent keeps 2.

### The start charge

- **API.** The type `StartCharge` (`new(history, pools)`, `history`, `pools`, `is_zero`; `#[non_exhaustive]`), re-exported from `revm_interpreter`, and `GasTracker::refill_start_charge`.
  `FrameStart::charge` (see Frame-start hook).
  `charged_start` on `CallInputs`, `CallOutcome` and `CreateOutcome` (fields) and on `CreateInputs` (`charged_start`, `set_charged_start`); `FrameResult::refundable_start_charge`.
  `revm_handler::execution::settle_first_frame`, the settlement of the first frame.
- **Contract.** The start charge is what a frame start charged its caller and the frame's failure gives back: history gas and an amount of each pool (`PoolAmounts`).
  The `frame_starting` hook charges the caller's tracker and declares the refundable part on the start it is handed (`start.charge`); the instruction copies it onto the frame's input when the hook admits the start; a start the hook refuses discards the charge with the start, and what the hook charged stands.
  The transaction's frame has no start hook: a consumer that charges the transaction's tracker for it in pre-execution puts the refundable part on the first frame's input (`charged_start`), from `Handler::first_frame_input`.
  The charge rides from the input to the outcome, and `return_result` refunds it to the caller whole, with `refill_start_charge` (`refill_history`, and `refill_pool` for each pool), on the condition `refundable_state_gas_charge` uses: a call that did not succeed, a creation that did not succeed or deployed no address; `last_frame_result` refunds the first frame's the same way, and after a halt consumes the regular gas the refill credited, as it does for the state refund.
  `last_frame_result` settles the first frame through `execution::settle_first_frame`, which is all of mainnet's settlement but the copy of the transaction's gas into the frame's result: the spend and the settlement of the frame, the EIP-2780 state-gas refund and the start-charge refund, each with its halt rule.
  A chain whose handler overrides `last_frame_result` calls it rather than copying the mainnet body, so the start-charge refund and any later step reach it too, and applies its own rules between it and the copy, such as the op-revm fork's pre-Regolith deposit rule.
  Upstream's own steps of the settlement (the spend, `handle_reservoir_remaining_gas` and the EIP-2780 refund) live in `crates/handler/src/execution/first_frame.rs`, not in `handler.rs`: a pick or a base-line merge that changes upstream's `last_frame_result` body lands there, and records the move as its conflict resolution.
  A frame that succeeds keeps the charge.
  A precompile's answer and a start the handler refuses before any frame (call depth, balance) carry the input's charge, so they refund as a frame does.
  An outcome an inspector builds to answer a start carries no charge unless the inspector copies the input's (`CallOutcome::new` and `CreateOutcome::new` carry none), so the caller keeps the charge, as revm keeps its own upfront state charge on an outcome a tool builds; an outcome the inspector rewrites at `call_end` or `create_end` keeps the frame's charge, so a success rewritten into a failure is refunded.
  An answer a consumer builds itself, such as an interceptor's, is refunded by the same rule: it carries the charge the consumer puts on it, and none when built with `new`.
  A consumer that settles a result into a caller's tracker itself refunds the charge with `refill_start_charge` on the same condition, so a pool added to the charge reaches it.
- **Consumer.** mega-evm charges a start's write records to the caller at the hook, after the forward and before the frame's input takes its share of the reservoir, and must give them back when the frame fails. It draws a start's records on the child's pools, so its hook declares history alone, but a creation's creator record is drawn from the caller's pools at the hook: when revm answers the creation before it bumps the nonce, the engine adds what the record drew to the answer's charge, so the creation's failure gives it back (`give_back_creator_record`). The refund belongs where the fork refunds the upfront state charge.
- **Removal.** When upstream carries an equivalent refundable start charge, or when mega-evm stops charging frame starts.
- **Default.** The hook declares nothing, so the fields and the refund are zero.
  The charge is four words, history and one per pool, on every input and outcome; its size is part of the outcomes' measured above (see Resource pools).
  `settle_first_frame` is inlined: where measured (callgrind, the fork's release profile), a transfer and a transaction of 64 calls run the same instruction count as with its steps written out in `last_frame_result`.
  A handler that overrides `last_frame_result` with a copy of mainnet's settlement instead of calling `settle_first_frame` compiles, but does not refund the first frame's start charge; that matters only to a consumer that declares one.
  This entry is an exception to rule 1 (`api:exception`): `CallInputs`, `CallOutcome` and `CreateOutcome` are constructible by literal and carry fields upstream's do not (`charged_start`, and on `CallInputs` also `pool_shares` and `history_allowance`), so a literal written for upstream's fields does not compile until it names them.
  A consumer that uses none of the figures names them at their defaults, no charge (`StartCharge::default()`), unlimited shares (`PoolAmounts::UNLIMITED`) and no allowance (`0`), and keeps upstream's behaviour.
  A consumer that answers a start with an outcome it builds itself sets the answer's charge as the Contract says and its pools as Resource pools says.
  The outcomes also carry the state-gas refund's `charged_state_gas_address` (see State-gas price), which upstream's do not, so an outcome literal written for upstream's fields does not compile against the fork whatever this entry adds; `CallInputs` has upstream's fields apart from this entry's, so this entry is what breaks its literals.
  `CreateInputs` has private fields and is built with `CreateInputs::new`, so its new fields break no literal.

### The history allowance

- **API.** On `GasTracker`, forwarded on `Gas`: `history_allowance`, `set_history_allowance`, `draw_history_allowance`.
  `GasId::call_history_allowance` and `GasParams::call_history_allowance()`; `history_allowance` on `CallInputs`.
- **Contract.** The allowance is history gas a frame may spend before it charges its budget.
  A call that transfers value gives its child the schedule's `call_history_allowance`, decided where the call instruction adds the call stipend, carried on the input and set on the child's tracker when it starts, on the frame and on an answer alike; no other start gives one, and the transaction's frame has none.
  Only a consumer draws it: `draw_history_allowance(cost)` takes `min(cost, allowance)` and returns the part of `cost` the allowance did not cover, which the consumer charges as history.
  A precompile's answer is given the input's allowance after its provider ran, so the allowance it carries back is the input's, whatever the provider drew.
  The allowance is the frame's alone: it is not returned, merged or rolled back.
- **Consumer.** mega-evm lets a value call's child log a bounded number of bytes without charging history, as the call stipend lets it run. Only the call instruction knows which starts get the stipend.
- **Removal.** When upstream carries an equivalent allowance, or when mega-evm drops the rule.
- **Default.** Zero on every schedule, so every child starts with none and a draw returns the whole cost; nothing in the workspace draws it.

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

## Baseline behaviour the fork changes

Picked upstream fixes change revm 40.0.3's results where 40.0.3 was wrong; that is why they are taken (see Tracking upstream), and each pick's pull request says what it changes.
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
Not taken: tooling-only changes, feature-gated additions the consumers do not use, and change/revert pairs, after checking that no taken commit depends on them.
Picks are batched by crate or subsystem, one pull request per batch, so one reviewer can read a whole batch.
The fork has no benchmark gate: a performance pick must leave execution results unchanged under the crate tests and the EEST run, and cite upstream's benchmark numbers in the pull request body.
A pick that touches a precompile is tested under mutually exclusive feature sets, because `--all-features` selects only the preferred backend in each `cfg_if!` block: `--no-default-features --features std` (every fallback: the arkworks BLS12-381 and KZG paths, `aurora-engine-modexp`, `k256`, `p256`), the default set, `--all-features`, and the single feature that selects the backend the pick changes (`std,blst`, `std,c-kzg`, `std,gmp`, `std,secp256k1` or `std,p256-aws-lc-rs`) on its own.
A pick that breaks the public API needs `api:exception`, the deviating items listed in the pull request body, and a check of every consumer that uses them.
An upstream change that upstream later reverted but the fork still needs lands as a fork-owned opt-in switch (`mega:hook`) whose default keeps upstream's behaviour.
A picked upstream fix that changes journal semantics lands with an opt-in switch back to the base line's behaviour when a consumer reproduces the base line exactly; the switch defaults to the picked behaviour, the consumer that needs the base line sets it, and nothing else reads it. `CfgEnv::legacy_journal_semantics` (the cold-slot re-baseline and the `CodeChange` revert of revm 40.0.3) is the first.
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
