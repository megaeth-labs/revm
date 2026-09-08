# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [19.0.0](https://github.com/megaeth-labs/revm/compare/revm-context-v18.0.3...revm-context-v19.0.0) - 2026-09-08

### Added

- Bake EIP-8037 CPSB into gas params ([#3714](https://github.com/megaeth-labs/revm/pull/3714))
- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/megaeth-labs/revm/pull/3667))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/megaeth-labs/revm/pull/3610))
- add EIP-8037 / TIP-1016 state gas support ([#3406](https://github.com/megaeth-labs/revm/pull/3406))
- add CallInput::as_bytes ([#3515](https://github.com/megaeth-labs/revm/pull/3515))
- add crate-level re-exports for all revm-* dependencies ([#3507](https://github.com/megaeth-labs/revm/pull/3507))
- Part of amsterdam devnet3 EIP updates ([#3438](https://github.com/megaeth-labs/revm/pull/3438))
- *(cfg)* add EIP-7708 configuration options ([#3395](https://github.com/megaeth-labs/revm/pull/3395))
- *(gas_params)* add configurable EIP-7702 auth refund ([#3366](https://github.com/megaeth-labs/revm/pull/3366))
- *(gas)* add tx_access_list_cost helper to GasParams ([#3349](https://github.com/megaeth-labs/revm/pull/3349))
- Implement EIP-7843 SLOTNUM opcode for Amsterdam ([#3340](https://github.com/megaeth-labs/revm/pull/3340))
- Implement EIP-7708 ETH transfers emit a log ([#3334](https://github.com/megaeth-labs/revm/pull/3334))
- new gas params, tx initial gas and codedeposit ([#3260](https://github.com/megaeth-labs/revm/pull/3260))
- move GasParams to Cfg ([#3229](https://github.com/megaeth-labs/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/megaeth-labs/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/megaeth-labs/revm/pull/3070))
- Gas params ([#3132](https://github.com/megaeth-labs/revm/pull/3132))
- JournaledAccount sload/sstore ([#3201](https://github.com/megaeth-labs/revm/pull/3201))
- Restrict Database::Error. JournaledAccountTr ([#3199](https://github.com/megaeth-labs/revm/pull/3199))
- Add set_nonce journal entry and fn ([#3163](https://github.com/megaeth-labs/revm/pull/3163))
- *(context)* add mark_cold method to JournaledAccount ([#3160](https://github.com/megaeth-labs/revm/pull/3160))
- generic Context::new ([#3156](https://github.com/megaeth-labs/revm/pull/3156))
- process precompile logs to inspector ([#3148](https://github.com/megaeth-labs/revm/pull/3148))
- selfdestruct oog on cold load ([#3140](https://github.com/megaeth-labs/revm/pull/3140))
- JournaledAccount, a nice way to update and track changes ([#3086](https://github.com/megaeth-labs/revm/pull/3086))
- dont load access list immediately ([#3116](https://github.com/megaeth-labs/revm/pull/3116))
- Support bubbling up first precompile error messages  ([#2905](https://github.com/megaeth-labs/revm/pull/2905))
- add transaction index to batch execution error handling ([#3000](https://github.com/megaeth-labs/revm/pull/3000))
- Add Str(Cow<'static, str>) to InvalidTransaction error enum ([#2998](https://github.com/megaeth-labs/revm/pull/2998))
- allow EIP-7623 to be disabled ([#2985](https://github.com/megaeth-labs/revm/pull/2985))
- Introduced `all_mut` and `all` functions to ContextTr ([#2992](https://github.com/megaeth-labs/revm/pull/2992))
- send bytecode with call input ([#2963](https://github.com/megaeth-labs/revm/pull/2963))
- *(op-revm)* Add an option to disable "fee-charge" on `op-revm` ([#2980](https://github.com/megaeth-labs/revm/pull/2980))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/megaeth-labs/revm/pull/2935))
- add generic state to ResultAndState ([#2897](https://github.com/megaeth-labs/revm/pull/2897))
- short address for journal cold/warm check ([#2849](https://github.com/megaeth-labs/revm/pull/2849))
- implement `Transaction` for `Either` ([#2662](https://github.com/megaeth-labs/revm/pull/2662))
- optional_eip3541 ([#2661](https://github.com/megaeth-labs/revm/pull/2661))
- remove EOF ([#2644](https://github.com/megaeth-labs/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/megaeth-labs/revm/pull/2611)) ([#2642](https://github.com/megaeth-labs/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- change blob_max_count to max_blobs_per_tx ([#2608](https://github.com/megaeth-labs/revm/pull/2608))
- add optional priority fee check configuration ([#2588](https://github.com/megaeth-labs/revm/pull/2588))
- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/megaeth-labs/revm/pull/2575))
- added TxEnv::new_bench() add util function ([#2556](https://github.com/megaeth-labs/revm/pull/2556))
- Config blob basefee fraction ([#2551](https://github.com/megaeth-labs/revm/pull/2551))
- expand timestamp/block_number to u256 ([#2546](https://github.com/megaeth-labs/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- make blob max number optional ([#2532](https://github.com/megaeth-labs/revm/pull/2532))
- add builder pattern for TxEnv ([#2518](https://github.com/megaeth-labs/revm/pull/2518))
- make Journal::set_code to be EIP-7702 zero address bytecode aware ([#2511](https://github.com/megaeth-labs/revm/pull/2511))
- *(Osaka)* disable EOF ([#2480](https://github.com/megaeth-labs/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/megaeth-labs/revm/pull/2462))
- Add a custom address to the CreateScheme. ([#2464](https://github.com/megaeth-labs/revm/pull/2464))
- *(Handler)* merge state validation with deduct_caller ([#2460](https://github.com/megaeth-labs/revm/pull/2460))
- add chain_ref method to ContextTr trait ([#2450](https://github.com/megaeth-labs/revm/pull/2450))
- *(tx)* Add Either RecoveredAuthorization ([#2448](https://github.com/megaeth-labs/revm/pull/2448))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/megaeth-labs/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/megaeth-labs/revm/pull/2382))
- cache precompile warming ([#2317](https://github.com/megaeth-labs/revm/pull/2317))
- Add JournalInner ([#2311](https://github.com/megaeth-labs/revm/pull/2311))
- InspectEvm fn renames, inspector docs, book cleanup ([#2275](https://github.com/megaeth-labs/revm/pull/2275))
- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/megaeth-labs/revm/pull/2233))
- allow reuse of API for calculating initial tx gas for tx ([#2215](https://github.com/megaeth-labs/revm/pull/2215))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- add custom error to context ([#2197](https://github.com/megaeth-labs/revm/pull/2197))
- Add tx/block to EvmExecution trait ([#2195](https://github.com/megaeth-labs/revm/pull/2195))
- added with_ref_db fn to Context ([#2164](https://github.com/megaeth-labs/revm/pull/2164))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- make journal entries generic ([#2154](https://github.com/megaeth-labs/revm/pull/2154))
- Standalone Host, remove default fn from context ([#2147](https://github.com/megaeth-labs/revm/pull/2147))
- add constructor with hardfork ([#2135](https://github.com/megaeth-labs/revm/pull/2135))
- implement AccessListTr for Vec ([#2136](https://github.com/megaeth-labs/revm/pull/2136))
- allow host to be implemented on custom context ([#2112](https://github.com/megaeth-labs/revm/pull/2112))
- add the debug impl for Evm and EvmData type ([#2126](https://github.com/megaeth-labs/revm/pull/2126))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- Split Inspector trait from EthHandler into standalone crate ([#2075](https://github.com/megaeth-labs/revm/pull/2075))
- Introduce Auth and AccessList traits ([#2079](https://github.com/megaeth-labs/revm/pull/2079))
- integrate alloy-eips ([#2078](https://github.com/megaeth-labs/revm/pull/2078))
- *(eip7702)* devnet6 changes and bump eest tests ([#2055](https://github.com/megaeth-labs/revm/pull/2055))
- Evm structure (Cached Instructions and Precompiles) ([#2049](https://github.com/megaeth-labs/revm/pull/2049))
- Context execution ([#2013](https://github.com/megaeth-labs/revm/pull/2013))
- EthHandler trait ([#2001](https://github.com/megaeth-labs/revm/pull/2001))
- *(EIP-7840)* Add blob schedule to execution client cfg ([#1980](https://github.com/megaeth-labs/revm/pull/1980))
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 ([#1969](https://github.com/megaeth-labs/revm/pull/1969))
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 ([#1965](https://github.com/megaeth-labs/revm/pull/1965))
- simplify Transaction trait ([#1959](https://github.com/megaeth-labs/revm/pull/1959))
- align Block trait ([#1957](https://github.com/megaeth-labs/revm/pull/1957))
- expose precompile address in Journal, DB::Error: StdError ([#1956](https://github.com/megaeth-labs/revm/pull/1956))
- Make Ctx journal generic ([#1933](https://github.com/megaeth-labs/revm/pull/1933))
- Restucturing Part7 Handler and Context rework ([#1865](https://github.com/megaeth-labs/revm/pull/1865))
- restructuring Part6 transaction crate ([#1814](https://github.com/megaeth-labs/revm/pull/1814))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))

### Fixed

- *(context)* use storage_by_account_id fast path in sload ([#3535](https://github.com/megaeth-labs/revm/pull/3535))
- make DummyHost return defaults instead of errors for storage ops ([#3503](https://github.com/megaeth-labs/revm/pull/3503))
- expose JournalLoadError from load_account_mut_skip_cold_load ([#3477](https://github.com/megaeth-labs/revm/pull/3477))
- *(context)* correct `ResultGas::final_refunded()` when floor gas is active ([#3450](https://github.com/megaeth-labs/revm/pull/3450))
- *(journal)* emit EIP-7708 log for selfdestructed accounts with remaining balance ([#3394](https://github.com/megaeth-labs/revm/pull/3394))
- use provided code hash when setting account code ([#3324](https://github.com/megaeth-labs/revm/pull/3324))
- fix API comment ([#3293](https://github.com/megaeth-labs/revm/pull/3293))
- *(test)* one gasid name is missing ([#3290](https://github.com/megaeth-labs/revm/pull/3290))
- set transaction_id on new account ([#3204](https://github.com/megaeth-labs/revm/pull/3204))
- use access list to decide if slot is cold ([#3149](https://github.com/megaeth-labs/revm/pull/3149))
- *(context)* avoid double reference in `Context::all()` ([#3131](https://github.com/megaeth-labs/revm/pull/3131))
- hook up Cfg::memory_limit ([#3129](https://github.com/megaeth-labs/revm/pull/3129))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- FrameStack mark push/end_init as unsafe ([#2929](https://github.com/megaeth-labs/revm/pull/2929))
- skip cold load on oog ([#2903](https://github.com/megaeth-labs/revm/pull/2903))
- EIP-7702 target check to return correct error ([#2896](https://github.com/megaeth-labs/revm/pull/2896))
- correct various typos in documentation and comments ([#2855](https://github.com/megaeth-labs/revm/pull/2855))
- swapped comments for db and db_mut methods in JournalTr trait ([#2774](https://github.com/megaeth-labs/revm/pull/2774))
- fully deprecate serde-json ([#2767](https://github.com/megaeth-labs/revm/pull/2767))
- fix typo: Rename is_created_globaly to is_created_globally ([#2692](https://github.com/megaeth-labs/revm/pull/2692))
- OpTransactionBuilder dont override envelope ([#2681](https://github.com/megaeth-labs/revm/pull/2681))
- call stack_frame.clear() at end ([#2656](https://github.com/megaeth-labs/revm/pull/2656))
- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/megaeth-labs/revm/pull/2581))
- use HashMap::default in LocalContext ([#2451](https://github.com/megaeth-labs/revm/pull/2451))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- Effective gas price should check tx type ([#2375](https://github.com/megaeth-labs/revm/pull/2375))
- remove duplicated load_account() ([#2225](https://github.com/megaeth-labs/revm/pull/2225))
- correct propagate features ([#2177](https://github.com/megaeth-labs/revm/pull/2177))
- clear JournalState and set first journal vec ([#1929](https://github.com/megaeth-labs/revm/pull/1929))
- Clear journal ([#1927](https://github.com/megaeth-labs/revm/pull/1927))
- *(revme)* include correct bytecode for snailtracer  ([#1917](https://github.com/megaeth-labs/revm/pull/1917))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))

### Other

- release prep — bump all crates with unpublished changes ([#3721](https://github.com/megaeth-labs/revm/pull/3721))
- release ([#3705](https://github.com/megaeth-labs/revm/pull/3705))
- v110 release prep ([#3702](https://github.com/megaeth-labs/revm/pull/3702))
- release ([#3701](https://github.com/megaeth-labs/revm/pull/3701))
- *(eip8037)* remove dead refill_amount tracking ([#3699](https://github.com/megaeth-labs/revm/pull/3699))
- v109 release prep ([#3695](https://github.com/megaeth-labs/revm/pull/3695))
- release ([#3679](https://github.com/megaeth-labs/revm/pull/3679))
- *(context)* centralize cfg-to-journal sync ([#3686](https://github.com/megaeth-labs/revm/pull/3686))
- change &mut self to &self for read-only methods ([#3669](https://github.com/megaeth-labs/revm/pull/3669))
- restructure `Journal` traits ([#3663](https://github.com/megaeth-labs/revm/pull/3663))
- use get instead of get_mut ([#3643](https://github.com/megaeth-labs/revm/pull/3643))
- remove unused spec ids ([#3649](https://github.com/megaeth-labs/revm/pull/3649))
- *(gas)* simplify log2floor ([#3629](https://github.com/megaeth-labs/revm/pull/3629))
- audit #[allow] attributes ([#3611](https://github.com/megaeth-labs/revm/pull/3611))
- backport v107 release notes from branch ([#3617](https://github.com/megaeth-labs/revm/pull/3617))
- remove pointer field from GasParams ([#3608](https://github.com/megaeth-labs/revm/pull/3608))
- [**breaking**] return Result from instruction functions ([#3558](https://github.com/megaeth-labs/revm/pull/3558))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/megaeth-labs/revm/pull/3592))
- no alloc for empty accounts ([#3590](https://github.com/megaeth-labs/revm/pull/3590))
- avoid cloning precompiles on warmup ([#3586](https://github.com/megaeth-labs/revm/pull/3586))
- pass reservoir into `first_frame_input` ([#3578](https://github.com/megaeth-labs/revm/pull/3578))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/megaeth-labs/revm/pull/3568))
- release ([#3472](https://github.com/megaeth-labs/revm/pull/3472))
- move EIP-8037 gas cap validation into validate_initial_tx_gas ([#3552](https://github.com/megaeth-labs/revm/pull/3552))
- deprecate set_spec and clean up deprecation attrs ([#3550](https://github.com/megaeth-labs/revm/pull/3550))
- add comment about frame stack initial capacity ([#3527](https://github.com/megaeth-labs/revm/pull/3527))
- use AnyError for PrecompileError::Fatal and EVMError::Custom ([#3502](https://github.com/megaeth-labs/revm/pull/3502))
- clarify PrecompileError::Fatal vs Other and EVMError::Custom ([#3496](https://github.com/megaeth-labs/revm/pull/3496))
- bump revm-database-interface to v10.0.0 and all dependents (v107) ([#3474](https://github.com/megaeth-labs/revm/pull/3474))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- move EIP-161 state clear into journal finalize ([#3444](https://github.com/megaeth-labs/revm/pull/3444))
- *(gas)* simplify log2floor implementation ([#3440](https://github.com/megaeth-labs/revm/pull/3440))
- [**breaking**] add logs to Revert and Halt variants of ExecutionResult ([#3424](https://github.com/megaeth-labs/revm/pull/3424))
- [**breaking**] add ResultGas struct to ExecutionResult ([#3413](https://github.com/megaeth-labs/revm/pull/3413))
- impl Display and Error for TxEnvBuildError and DeriveTxTypeError ([#3409](https://github.com/megaeth-labs/revm/pull/3409))
- [**breaking**] flatten Bytecode ([#3375](https://github.com/megaeth-labs/revm/pull/3375))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- mark journal entry functions as deprecated ([#3367](https://github.com/megaeth-labs/revm/pull/3367))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/megaeth-labs/revm/pull/3358))
- *(handler)* extract duplicate ContextError handling ([#3312](https://github.com/megaeth-labs/revm/pull/3312))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
- *(gas_params)* add dedicated GasIds for sstore_refund ([#3310](https://github.com/megaeth-labs/revm/pull/3310))
- remove redundant clones in gas params defaults ([#3300](https://github.com/megaeth-labs/revm/pull/3300))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/megaeth-labs/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/megaeth-labs/revm/pull/3272))
- add Display trait for ExecutionResult and related types ([#3267](https://github.com/megaeth-labs/revm/pull/3267))
- add Display for HaltReason and OutOfGasError ([#3265](https://github.com/megaeth-labs/revm/pull/3265))
- *(cleanup)* remove unused duplicate ContextSetters trait in context crate ([#3225](https://github.com/megaeth-labs/revm/pull/3225))
- *(fmt)* merge all imports ([#3184](https://github.com/megaeth-labs/revm/pull/3184))
- release ([#3162](https://github.com/megaeth-labs/revm/pull/3162))
- tag v100 revm v33.0.0 ([#3161](https://github.com/megaeth-labs/revm/pull/3161))
- release ([#3136](https://github.com/megaeth-labs/revm/pull/3136))
- merge v98 versions bumps ([#3155](https://github.com/megaeth-labs/revm/pull/3155))
- release ([#3113](https://github.com/megaeth-labs/revm/pull/3113))
- journal transfer fn cleanup ([#3085](https://github.com/megaeth-labs/revm/pull/3085))
- release ([#3102](https://github.com/megaeth-labs/revm/pull/3102))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- resize short addresses bitvec instead of reallocating ([#3083](https://github.com/megaeth-labs/revm/pull/3083))
- bump minor versions ([#3078](https://github.com/megaeth-labs/revm/pull/3078))
- release ([#3061](https://github.com/megaeth-labs/revm/pull/3061))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- make precompile error pub ([#3057](https://github.com/megaeth-labs/revm/pull/3057))
- changelog update for v87 ([#3056](https://github.com/megaeth-labs/revm/pull/3056))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- helper function gas_balance_spending ([#3030](https://github.com/megaeth-labs/revm/pull/3030))
- remove unreachable zero-denominator check in fake_exponential ([#3039](https://github.com/megaeth-labs/revm/pull/3039))
- add ensure_enough_balance helper ([#3033](https://github.com/megaeth-labs/revm/pull/3033))
- add default impl for tx_local_mut and tx_journal_mut ([#3029](https://github.com/megaeth-labs/revm/pull/3029))
- *(op-revm)* propagate optional_fee_charge feature ([#3020](https://github.com/megaeth-labs/revm/pull/3020))
- prealloc few frames ([#2965](https://github.com/megaeth-labs/revm/pull/2965))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/megaeth-labs/revm/pull/2925))
- cargo update ([#2930](https://github.com/megaeth-labs/revm/pull/2930))
- release ([#2899](https://github.com/megaeth-labs/revm/pull/2899))
- skip drain if checkpoing is inconsistent ([#2911](https://github.com/megaeth-labs/revm/pull/2911))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- Aggregate changes from PRs #2866, #2867, and #2874 ([#2876](https://github.com/megaeth-labs/revm/pull/2876))
- make ci happy ([#2863](https://github.com/megaeth-labs/revm/pull/2863))
- tag v84 revm v28.0.0 ([#2856](https://github.com/megaeth-labs/revm/pull/2856))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- rm redundant lifetime constraints ([#2850](https://github.com/megaeth-labs/revm/pull/2850))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- un-Box frames ([#2761](https://github.com/megaeth-labs/revm/pull/2761))
- discard generic host implementation ([#2738](https://github.com/megaeth-labs/revm/pull/2738))
- release ([#2682](https://github.com/megaeth-labs/revm/pull/2682))
- add comprehensive tests for TxEnvBuilder ([#2690](https://github.com/megaeth-labs/revm/pull/2690))
- tag v81 revm v27.0.1 ([#2689](https://github.com/megaeth-labs/revm/pull/2689))
- v80 revm v27.0.1 ([#2683](https://github.com/megaeth-labs/revm/pull/2683))
- tag v79 revm v27.0.0 ([#2680](https://github.com/megaeth-labs/revm/pull/2680))
- release ([#2659](https://github.com/megaeth-labs/revm/pull/2659))
- use TxEnv::builder ([#2652](https://github.com/megaeth-labs/revm/pull/2652))
- fix copy-pasted inner doc comments ([#2663](https://github.com/megaeth-labs/revm/pull/2663))
- release ([#2657](https://github.com/megaeth-labs/revm/pull/2657))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- bump all deps ([#2647](https://github.com/megaeth-labs/revm/pull/2647))
- include local context as generic ([#2645](https://github.com/megaeth-labs/revm/pull/2645))
- re-use frame allocation ([#2636](https://github.com/megaeth-labs/revm/pull/2636))
- store coinbase address separately to avoid cloning warm addresses in the common case ([#2634](https://github.com/megaeth-labs/revm/pull/2634))
- optimize warm_preloaded_addresses reset ([#2625](https://github.com/megaeth-labs/revm/pull/2625))
- rename `transact` methods ([#2616](https://github.com/megaeth-labs/revm/pull/2616))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- tag v75 revm v24.0.1 ([#2563](https://github.com/megaeth-labs/revm/pull/2563)) ([#2589](https://github.com/megaeth-labs/revm/pull/2589))
- support functions for eip7918 ([#2579](https://github.com/megaeth-labs/revm/pull/2579))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/megaeth-labs/revm/pull/2568))
- *(docs)* context crate lints ([#2565](https://github.com/megaeth-labs/revm/pull/2565))
- unify calling of journal account loading ([#2561](https://github.com/megaeth-labs/revm/pull/2561))
- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/megaeth-labs/revm/pull/2560))
- *(cfg)* add tx_chain_id_check fields. Optimize effective gas cost calc ([#2557](https://github.com/megaeth-labs/revm/pull/2557))
- add dot to trigger ci ([#2552](https://github.com/megaeth-labs/revm/pull/2552))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- add TxEnvBuilder::build_fill ([#2536](https://github.com/megaeth-labs/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/megaeth-labs/revm/pull/2461))
- tag v71, revm v23.1.0 semver major bump ([#2492](https://github.com/megaeth-labs/revm/pull/2492))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- typos ([#2474](https://github.com/megaeth-labs/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- remove default capacity on journal reverts ([#2449](https://github.com/megaeth-labs/revm/pull/2449))
- *(journal)* flatten journal entries ([#2440](https://github.com/megaeth-labs/revm/pull/2440))
- clone_from precompile addresses ([#2438](https://github.com/megaeth-labs/revm/pull/2438))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/megaeth-labs/revm/pull/2415)) ([#2416](https://github.com/megaeth-labs/revm/pull/2416))
- *(lints)* revm-context lints ([#2404](https://github.com/megaeth-labs/revm/pull/2404))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- make blob params u64 ([#2385](https://github.com/megaeth-labs/revm/pull/2385))
- set gas_priority_fee to None in TxEnv ([#2371](https://github.com/megaeth-labs/revm/pull/2371))
- tag v67 revm v21.0.0 ([#2341](https://github.com/megaeth-labs/revm/pull/2341))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- Remove LATEST variant from SpecId enum ([#2299](https://github.com/megaeth-labs/revm/pull/2299))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- v65 revm: v20.0.0-alpha.7 ([#2280](https://github.com/megaeth-labs/revm/pull/2280))
- remove wrong `&mut` and duplicated spec ([#2276](https://github.com/megaeth-labs/revm/pull/2276))
- Add custom instruction example ([#2261](https://github.com/megaeth-labs/revm/pull/2261))
- fix clippy ([#2238](https://github.com/megaeth-labs/revm/pull/2238))
- use AccessListItem associated type instead of AccessList ([#2214](https://github.com/megaeth-labs/revm/pull/2214))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v62 revm v20.0.0-alpha.5 ([#2198](https://github.com/megaeth-labs/revm/pull/2198))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- Add comments to handler methods ([#2188](https://github.com/megaeth-labs/revm/pull/2188))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- pre EIP-7702 does not need to load code ([#2162](https://github.com/megaeth-labs/revm/pull/2162))
- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/megaeth-labs/revm/pull/2155))
- rename transact_previous to replay, move EvmTr traits ([#2153](https://github.com/megaeth-labs/revm/pull/2153))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- move mainnet builder to handler crate ([#2138](https://github.com/megaeth-labs/revm/pull/2138))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/megaeth-labs/revm/pull/2132))
- Adding function derive_tx_type to TxEnv ([#2118](https://github.com/megaeth-labs/revm/pull/2118))
- fix eofcreate error typo ([#2120](https://github.com/megaeth-labs/revm/pull/2120))
- remove wrong `&mut`/`TODO`, and avoid useless `get_mut` ([#2111](https://github.com/megaeth-labs/revm/pull/2111))
- Add docs to revm-bytecode crate ([#2108](https://github.com/megaeth-labs/revm/pull/2108))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/megaeth-labs/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/megaeth-labs/revm/pull/2088))
- rm database from context-interface ([#2087](https://github.com/megaeth-labs/revm/pull/2087))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- Rename NameTrait to NameTr ([#2084](https://github.com/megaeth-labs/revm/pull/2084))
- API cleanup ([#2067](https://github.com/megaeth-labs/revm/pull/2067))
- Add helpers with_inspector with_precompile ([#2063](https://github.com/megaeth-labs/revm/pull/2063))
- relax halt reason bounds ([#2041](https://github.com/megaeth-labs/revm/pull/2041))
- simplify some generics ([#2032](https://github.com/megaeth-labs/revm/pull/2032))
- Add helper functions for JournalInit #1879 ([#1961](https://github.com/megaeth-labs/revm/pull/1961))
- fix journal naming for inc/dec balance ([#1976](https://github.com/megaeth-labs/revm/pull/1976))
- Make inspector use generics, rm associated types ([#1934](https://github.com/megaeth-labs/revm/pull/1934))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
- tie journal database with database getter ([#1923](https://github.com/megaeth-labs/revm/pull/1923))
- Move CfgEnv from context-interface to context crate ([#1910](https://github.com/megaeth-labs/revm/pull/1910))
- Update README.md examples section ([#1853](https://github.com/megaeth-labs/revm/pull/1853))
- Bump new logo ([#1735](https://github.com/megaeth-labs/revm/pull/1735))
- *(README)* add rbuilder to used-by ([#1585](https://github.com/megaeth-labs/revm/pull/1585))
- added simular to used-by ([#1521](https://github.com/megaeth-labs/revm/pull/1521))
- add Trin to used by list ([#1393](https://github.com/megaeth-labs/revm/pull/1393))
- Fix typo in readme ([#1185](https://github.com/megaeth-labs/revm/pull/1185))
- Add Hardhat to the "Used by" list ([#1164](https://github.com/megaeth-labs/revm/pull/1164))
- Add VERBS to used by list ([#1141](https://github.com/megaeth-labs/revm/pull/1141))
- license date and revm docs ([#1080](https://github.com/megaeth-labs/revm/pull/1080))
- *(docs)* Update the benchmark docs to point to revm package ([#906](https://github.com/megaeth-labs/revm/pull/906))
- *(docs)* Update top-level benchmark docs ([#894](https://github.com/megaeth-labs/revm/pull/894))
- clang requirement ([#784](https://github.com/megaeth-labs/revm/pull/784))
- Readme Updates ([#756](https://github.com/megaeth-labs/revm/pull/756))
- Logo ([#743](https://github.com/megaeth-labs/revm/pull/743))
- book workflow ([#537](https://github.com/megaeth-labs/revm/pull/537))
- add example to revm crate ([#468](https://github.com/megaeth-labs/revm/pull/468))
- Update README.md ([#424](https://github.com/megaeth-labs/revm/pull/424))
- add no_std to primitives ([#366](https://github.com/megaeth-labs/revm/pull/366))
- revm-precompiles to revm-precompile
- Bump v20, changelog ([#350](https://github.com/megaeth-labs/revm/pull/350))
- typos ([#232](https://github.com/megaeth-labs/revm/pull/232))
- Add support for old forks. ([#191](https://github.com/megaeth-labs/revm/pull/191))
- revm bump 1.8. update libs. snailtracer rename ([#159](https://github.com/megaeth-labs/revm/pull/159))
- typo fixes
- fix readme typo
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/megaeth-labs/revm/pull/52))
- readme. debuger update
- Bump revm v0.3.0. README updated
- readme
- Add time elapsed for tests
- readme updated
- Include Basefee into cost calc. readme change
- Initialize precompile accounts
- Status update. Taking a break
- Merkle calc. Tweaks and debugging for eip158
- Replace aurora bn lib with parity's. All Bn128Add/Mul/Pair tests passes
- TEMP
- one tab removed
- readme
- README Example simplified
- Gas calculation for Call/Create. Example Added
- readme usage
- README changes
- Static gas cost added
- Subroutine changelogs and reverts
- Readme postulates
- Spelling
- Restructure project
- First iteration. Machine is looking okay

## [18.0.3](https://github.com/bluealloy/revm/compare/revm-context-v18.0.2...revm-context-v18.0.3) - 2026-05-26

### Added

- Bake EIP-8037 CPSB into gas params ([#3714](https://github.com/bluealloy/revm/pull/3714))

## [18.0.2](https://github.com/bluealloy/revm/compare/revm-context-v18.0.1...revm-context-v18.0.2) - 2026-05-22

### Other

- updated the following local packages: revm-database-interface, revm-context-interface

## [18.0.1](https://github.com/bluealloy/revm/compare/revm-context-v18.0.0...revm-context-v18.0.1) - 2026-05-21

### Other

- updated the following local packages: revm-context-interface

## [18.0.0](https://github.com/bluealloy/revm/compare/revm-context-v16.0.1...revm-context-v18.0.0) - 2026-05-19

### Added

- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/bluealloy/revm/pull/3667))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/bluealloy/revm/pull/3610))

### Fixed

- *(context)* use storage_by_account_id fast path in sload ([#3535](https://github.com/bluealloy/revm/pull/3535))

### Other

- *(context)* centralize cfg-to-journal sync ([#3686](https://github.com/bluealloy/revm/pull/3686))
- change &mut self to &self for read-only methods ([#3669](https://github.com/bluealloy/revm/pull/3669))
- restructure `Journal` traits ([#3663](https://github.com/bluealloy/revm/pull/3663))
- use get instead of get_mut ([#3643](https://github.com/bluealloy/revm/pull/3643))
- remove unused spec ids ([#3649](https://github.com/bluealloy/revm/pull/3649))
- *(gas)* simplify log2floor ([#3629](https://github.com/bluealloy/revm/pull/3629))
- audit #[allow] attributes ([#3611](https://github.com/bluealloy/revm/pull/3611))
- backport v107 release notes from branch ([#3617](https://github.com/bluealloy/revm/pull/3617))
- remove pointer field from GasParams ([#3608](https://github.com/bluealloy/revm/pull/3608))
- [**breaking**] return Result from instruction functions ([#3558](https://github.com/bluealloy/revm/pull/3558))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/bluealloy/revm/pull/3592))
- no alloc for empty accounts ([#3590](https://github.com/bluealloy/revm/pull/3590))
- avoid cloning precompiles on warmup ([#3586](https://github.com/bluealloy/revm/pull/3586))
- pass reservoir into `first_frame_input` ([#3578](https://github.com/bluealloy/revm/pull/3578))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [16.0.1](https://github.com/bluealloy/revm/compare/revm-context-v16.0.0...revm-context-v16.0.1) - 2026-04-17

### Other

- updated the following local packages: revm-state, revm-context-interface, revm-database-interface

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-context-v15.0.0...revm-context-v16.0.0) - 2026-04-10

### Added

- add EIP-8037 / TIP-1016 state gas support ([#3406](https://github.com/bluealloy/revm/pull/3406))
- add CallInput::as_bytes ([#3515](https://github.com/bluealloy/revm/pull/3515))
- add crate-level re-exports for all revm-* dependencies ([#3507](https://github.com/bluealloy/revm/pull/3507))
- Part of amsterdam devnet3 EIP updates ([#3438](https://github.com/bluealloy/revm/pull/3438))

### Fixed

- make DummyHost return defaults instead of errors for storage ops ([#3503](https://github.com/bluealloy/revm/pull/3503))
- expose JournalLoadError from load_account_mut_skip_cold_load ([#3477](https://github.com/bluealloy/revm/pull/3477))

### Other

- move EIP-8037 gas cap validation into validate_initial_tx_gas ([#3552](https://github.com/bluealloy/revm/pull/3552))
- deprecate set_spec and clean up deprecation attrs ([#3550](https://github.com/bluealloy/revm/pull/3550))
- add comment about frame stack initial capacity ([#3527](https://github.com/bluealloy/revm/pull/3527))
- use AnyError for PrecompileError::Fatal and EVMError::Custom ([#3502](https://github.com/bluealloy/revm/pull/3502))
- clarify PrecompileError::Fatal vs Other and EVMError::Custom ([#3496](https://github.com/bluealloy/revm/pull/3496))

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-context-v14.0.0...revm-context-v15.0.0) - 2026-03-04

### Other

- bump revm-database-interface to v10.0.0

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-context-v13.0.0...revm-context-v14.0.0) - 2026-03-02

### Added

- *(cfg)* add EIP-7708 configuration options ([#3395](https://github.com/bluealloy/revm/pull/3395))
- *(gas_params)* add configurable EIP-7702 auth refund ([#3366](https://github.com/bluealloy/revm/pull/3366))
- *(gas)* add tx_access_list_cost helper to GasParams ([#3349](https://github.com/bluealloy/revm/pull/3349))
- Implement EIP-7843 SLOTNUM opcode for Amsterdam ([#3340](https://github.com/bluealloy/revm/pull/3340))
- Implement EIP-7708 ETH transfers emit a log ([#3334](https://github.com/bluealloy/revm/pull/3334))

### Fixed

- *(context)* correct `ResultGas::final_refunded()` when floor gas is active ([#3450](https://github.com/bluealloy/revm/pull/3450))
- *(journal)* emit EIP-7708 log for selfdestructed accounts with remaining balance ([#3394](https://github.com/bluealloy/revm/pull/3394))

### Other

- move EIP-161 state clear into journal finalize ([#3444](https://github.com/bluealloy/revm/pull/3444))
- *(gas)* simplify log2floor implementation ([#3440](https://github.com/bluealloy/revm/pull/3440))
- [**breaking**] add logs to Revert and Halt variants of ExecutionResult ([#3424](https://github.com/bluealloy/revm/pull/3424))
- [**breaking**] add ResultGas struct to ExecutionResult ([#3413](https://github.com/bluealloy/revm/pull/3413))
- impl Display and Error for TxEnvBuildError and DeriveTxTypeError ([#3409](https://github.com/bluealloy/revm/pull/3409))
- [**breaking**] flatten Bytecode ([#3375](https://github.com/bluealloy/revm/pull/3375))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))
- mark journal entry functions as deprecated ([#3367](https://github.com/bluealloy/revm/pull/3367))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))
- *(handler)* extract duplicate ContextError handling ([#3312](https://github.com/bluealloy/revm/pull/3312))

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-context-v12.1.0...revm-context-v13.0.0) - 2026-01-15

### Added

- new gas params, tx initial gas and codedeposit ([#3260](https://github.com/bluealloy/revm/pull/3260))
- move GasParams to Cfg ([#3229](https://github.com/bluealloy/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/bluealloy/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/bluealloy/revm/pull/3070))
- Gas params ([#3132](https://github.com/bluealloy/revm/pull/3132))
- JournaledAccount sload/sstore ([#3201](https://github.com/bluealloy/revm/pull/3201))
- Restrict Database::Error. JournaledAccountTr ([#3199](https://github.com/bluealloy/revm/pull/3199))

### Fixed

- fix API comment ([#3293](https://github.com/bluealloy/revm/pull/3293))
- *(test)* one gasid name is missing ([#3290](https://github.com/bluealloy/revm/pull/3290))
- set transaction_id on new account ([#3204](https://github.com/bluealloy/revm/pull/3204))

### Other

- *(gas_params)* add dedicated GasIds for sstore_refund ([#3310](https://github.com/bluealloy/revm/pull/3310))
- remove redundant clones in gas params defaults ([#3300](https://github.com/bluealloy/revm/pull/3300))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- add Display trait for ExecutionResult and related types ([#3267](https://github.com/bluealloy/revm/pull/3267))
- add Display for HaltReason and OutOfGasError ([#3265](https://github.com/bluealloy/revm/pull/3265))
- *(cleanup)* remove unused duplicate ContextSetters trait in context crate ([#3225](https://github.com/bluealloy/revm/pull/3225))
- *(fmt)* merge all imports ([#3184](https://github.com/bluealloy/revm/pull/3184))

## [12.1.0](https://github.com/bluealloy/revm/compare/revm-context-v12.0.0...revm-context-v12.1.0) - 2025-11-14

### Added

- Add set_nonce journal entry and fn ([#3163](https://github.com/bluealloy/revm/pull/3163))

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-context-v11.0.2...revm-context-v12.0.0) - 2025-11-10

### Added

- generic Context::new ([#3156](https://github.com/bluealloy/revm/pull/3156))
- process precompile logs to inspector ([#3148](https://github.com/bluealloy/revm/pull/3148))
- selfdestruct oog on cold load ([#3140](https://github.com/bluealloy/revm/pull/3140))

### Fixed

- use access list to decide if slot is cold ([#3149](https://github.com/bluealloy/revm/pull/3149))

### Other

- merge v98 versions bumps ([#3155](https://github.com/bluealloy/revm/pull/3155))

## [11.0.2](https://github.com/bluealloy/revm/compare/revm-context-v11.0.1...revm-context-v11.0.2) - 2025-11-10

### Other

- updated the following local packages: revm-database

## [11.0.1](https://github.com/bluealloy/revm/compare/revm-context-v11.0.0...revm-context-v11.0.1) - 2025-11-07

### Other

- add test

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-context-v10.1.2...revm-context-v11.0.0) - 2025-10-30

### Added

- JournaledAccount, a nice way to update and track changes ([#3086](https://github.com/bluealloy/revm/pull/3086))
- dont load access list immediately ([#3116](https://github.com/bluealloy/revm/pull/3116))

### Fixed

- *(context)* avoid double reference in `Context::all()` ([#3131](https://github.com/bluealloy/revm/pull/3131))
- hook up Cfg::memory_limit ([#3129](https://github.com/bluealloy/revm/pull/3129))

### Other

- journal transfer fn cleanup ([#3085](https://github.com/bluealloy/revm/pull/3085))

## [10.1.2](https://github.com/bluealloy/revm/compare/revm-context-v10.1.1...revm-context-v10.1.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface

## [10.1.1](https://github.com/bluealloy/revm/compare/revm-context-v10.1.0...revm-context-v10.1.1) - 2025-10-15

### Other

- resize short addresses bitvec instead of reallocating ([#3083](https://github.com/bluealloy/revm/pull/3083))

## [10.1.0](https://github.com/bluealloy/revm/compare/revm-context-v10.0.0...revm-context-v10.1.0) - 2025-10-09

### Other

- updated the following local packages: revm-database-interface, revm-database, revm-context-interface

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-context-v9.1.0...revm-context-v10.0.0) - 2025-10-07

### Added

- Support bubbling up first precompile error messages  ([#2905](https://github.com/bluealloy/revm/pull/2905))
- add transaction index to batch execution error handling ([#3000](https://github.com/bluealloy/revm/pull/3000))
- Add Str(Cow<'static, str>) to InvalidTransaction error enum ([#2998](https://github.com/bluealloy/revm/pull/2998))
- allow EIP-7623 to be disabled ([#2985](https://github.com/bluealloy/revm/pull/2985))
- Introduced `all_mut` and `all` functions to ContextTr ([#2992](https://github.com/bluealloy/revm/pull/2992))
- send bytecode with call input ([#2963](https://github.com/bluealloy/revm/pull/2963))
- *(op-revm)* Add an option to disable "fee-charge" on `op-revm` ([#2980](https://github.com/bluealloy/revm/pull/2980))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))
- FrameStack mark push/end_init as unsafe ([#2929](https://github.com/bluealloy/revm/pull/2929))
- skip cold load on oog ([#2903](https://github.com/bluealloy/revm/pull/2903))

### Other

- make precompile error pub ([#3057](https://github.com/bluealloy/revm/pull/3057))
- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- helper function gas_balance_spending ([#3030](https://github.com/bluealloy/revm/pull/3030))
- remove unreachable zero-denominator check in fake_exponential ([#3039](https://github.com/bluealloy/revm/pull/3039))
- add ensure_enough_balance helper ([#3033](https://github.com/bluealloy/revm/pull/3033))
- add default impl for tx_local_mut and tx_journal_mut ([#3029](https://github.com/bluealloy/revm/pull/3029))
- *(op-revm)* propagate optional_fee_charge feature ([#3020](https://github.com/bluealloy/revm/pull/3020))
- prealloc few frames ([#2965](https://github.com/bluealloy/revm/pull/2965))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/bluealloy/revm/pull/2925))
- cargo update ([#2930](https://github.com/bluealloy/revm/pull/2930))

## [9.1.0](https://github.com/bluealloy/revm/compare/revm-context-v9.0.2...revm-context-v9.1.0) - 2025-09-23

### Added

- *(op-revm)* Add an option to disable "fee-charge" on `op-revm` ([#2980](https://github.com/bluealloy/revm/pull/2980))

## [9.0.2](https://github.com/bluealloy/revm/compare/revm-context-v9.0.1...revm-context-v9.0.2) - 2025-08-23

### Fixed

- EIP-7702 target check to return correct error ([#2896](https://github.com/bluealloy/revm/pull/2896))

### Other

- skip drain if checkpoing is inconsistent ([#2911](https://github.com/bluealloy/revm/pull/2911))

## [9.0.1](https://github.com/bluealloy/revm/compare/revm-context-v9.0.0...revm-context-v9.0.1) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-context-interface, revm-database, revm-database-interface

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-context-v8.0.4...revm-context-v9.0.0) - 2025-08-06

### Added

- short address for journal cold/warm check ([#2849](https://github.com/bluealloy/revm/pull/2849))

### Fixed

- correct various typos in documentation and comments ([#2855](https://github.com/bluealloy/revm/pull/2855))

### Other

- rm redundant lifetime constraints ([#2850](https://github.com/bluealloy/revm/pull/2850))
- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))

## [8.0.4](https://github.com/bluealloy/revm/compare/revm-context-v8.0.3...revm-context-v8.0.4) - 2025-07-23

### Fixed

- fully deprecate serde-json ([#2767](https://github.com/bluealloy/revm/pull/2767))

### Other

- un-Box frames ([#2761](https://github.com/bluealloy/revm/pull/2761))
- discard generic host implementation ([#2738](https://github.com/bluealloy/revm/pull/2738))

## [8.0.3](https://github.com/bluealloy/revm/compare/revm-context-v8.0.2...revm-context-v8.0.3) - 2025-07-14

### Fixed

- fix typo: Rename is_created_globaly to is_created_globally ([#2692](https://github.com/bluealloy/revm/pull/2692))

### Other

- add comprehensive tests for TxEnvBuilder ([#2690](https://github.com/bluealloy/revm/pull/2690))

## [8.0.2](https://github.com/bluealloy/revm/compare/revm-context-v8.0.1...revm-context-v8.0.2) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-context-v7.0.1...revm-context-v8.0.1) - 2025-06-30

### Added

- implement `Transaction` for `Either` ([#2662](https://github.com/bluealloy/revm/pull/2662))
- optional_eip3541 ([#2661](https://github.com/bluealloy/revm/pull/2661))

### Other

- use TxEnv::builder ([#2652](https://github.com/bluealloy/revm/pull/2652))
- fix copy-pasted inner doc comments ([#2663](https://github.com/bluealloy/revm/pull/2663))

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-context-v7.0.0...revm-context-v7.0.1) - 2025-06-20

### Fixed

- call stack_frame.clear() at end ([#2656](https://github.com/bluealloy/revm/pull/2656))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-context-v6.0.0...revm-context-v7.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/bluealloy/revm/pull/2611)) ([#2642](https://github.com/bluealloy/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- change blob_max_count to max_blobs_per_tx ([#2608](https://github.com/bluealloy/revm/pull/2608))
- add optional priority fee check configuration ([#2588](https://github.com/bluealloy/revm/pull/2588))

### Other

- bump all deps ([#2647](https://github.com/bluealloy/revm/pull/2647))
- include local context as generic ([#2645](https://github.com/bluealloy/revm/pull/2645))
- re-use frame allocation ([#2636](https://github.com/bluealloy/revm/pull/2636))
- store coinbase address separately to avoid cloning warm addresses in the common case ([#2634](https://github.com/bluealloy/revm/pull/2634))
- optimize warm_preloaded_addresses reset ([#2625](https://github.com/bluealloy/revm/pull/2625))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-context-v5.0.1...revm-context-v6.0.0) - 2025-06-06

### Added

- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/bluealloy/revm/pull/2575))
- added TxEnv::new_bench() add util function ([#2556](https://github.com/bluealloy/revm/pull/2556))
- Config blob basefee fraction ([#2551](https://github.com/bluealloy/revm/pull/2551))
- expand timestamp/block_number to u256 ([#2546](https://github.com/bluealloy/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Fixed

- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/bluealloy/revm/pull/2581))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))
- support functions for eip7918 ([#2579](https://github.com/bluealloy/revm/pull/2579))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/bluealloy/revm/pull/2568))
- *(docs)* context crate lints ([#2565](https://github.com/bluealloy/revm/pull/2565))
- unify calling of journal account loading ([#2561](https://github.com/bluealloy/revm/pull/2561))
- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/bluealloy/revm/pull/2560))
- *(cfg)* add tx_chain_id_check fields. Optimize effective gas cost calc ([#2557](https://github.com/bluealloy/revm/pull/2557))
- add dot to trigger ci ([#2552](https://github.com/bluealloy/revm/pull/2552))

## [5.0.1](https://github.com/bluealloy/revm/compare/revm-context-v5.0.0...revm-context-v5.0.1) - 2025-05-31

### Other

- unify calling of journal account loading

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-context-v4.1.0...revm-context-v5.0.0) - 2025-05-22

### Added

- make blob max number optional ([#2532](https://github.com/bluealloy/revm/pull/2532))
- add builder pattern for TxEnv ([#2518](https://github.com/bluealloy/revm/pull/2518))
- make Journal::set_code to be EIP-7702 zero address bytecode aware ([#2511](https://github.com/bluealloy/revm/pull/2511))

### Other

- add TxEnvBuilder::build_fill ([#2536](https://github.com/bluealloy/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [4.1.0](https://github.com/bluealloy/revm/compare/revm-context-v4.0.0...revm-context-v4.1.0) - 2025-05-07

Dependency bump

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-context-v3.0.1...revm-context-v4.0.0) - 2025-05-07

### Added

- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/bluealloy/revm/pull/2462))
- Add a custom address to the CreateScheme. ([#2464](https://github.com/bluealloy/revm/pull/2464))
- *(Handler)* merge state validation with deduct_caller ([#2460](https://github.com/bluealloy/revm/pull/2460))
- add chain_ref method to ContextTr trait ([#2450](https://github.com/bluealloy/revm/pull/2450))
- *(tx)* Add Either RecoveredAuthorization ([#2448](https://github.com/bluealloy/revm/pull/2448))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/bluealloy/revm/pull/2382))

### Fixed

- use HashMap::default in LocalContext ([#2451](https://github.com/bluealloy/revm/pull/2451))

### Other

- typos ([#2474](https://github.com/bluealloy/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- remove default capacity on journal reverts ([#2449](https://github.com/bluealloy/revm/pull/2449))
- *(journal)* flatten journal entries ([#2440](https://github.com/bluealloy/revm/pull/2440))
- clone_from precompile addresses ([#2438](https://github.com/bluealloy/revm/pull/2438))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/bluealloy/revm/pull/2415)) ([#2416](https://github.com/bluealloy/revm/pull/2416))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-context-v3.0.0...revm-context-v3.0.1) - 2025-04-15

### Other

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-context-v2.0.0...revm-context-v3.0.0) - 2025-04-09

### Fixed

- Effective gas price should check tx type ([#2375](https://github.com/bluealloy/revm/pull/2375))

### Other

- make blob params u64 ([#2385](https://github.com/bluealloy/revm/pull/2385))
- set gas_priority_fee to None in TxEnv ([#2371](https://github.com/bluealloy/revm/pull/2371))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0...revm-context-v2.0.0) - 2025-03-28

### Added

- cache precompile warming ([#2317](https://github.com/bluealloy/revm/pull/2317))
- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))

### Other

- Remove LATEST variant from SpecId enum ([#2299](https://github.com/bluealloy/revm/pull/2299))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.6...revm-context-v1.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database

## [1.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.5...revm-context-v1.0.0-alpha.6) - 2025-03-21

### Added

- InspectEvm fn renames, inspector docs, book cleanup ([#2275](https://github.com/bluealloy/revm/pull/2275))

### Fixed

- remove duplicated load_account() ([#2225](https://github.com/bluealloy/revm/pull/2225))

### Other

- remove wrong `&mut` and duplicated spec ([#2276](https://github.com/bluealloy/revm/pull/2276))
- Add custom instruction example ([#2261](https://github.com/bluealloy/revm/pull/2261))
- fix clippy ([#2238](https://github.com/bluealloy/revm/pull/2238))
- use AccessListItem associated type instead of AccessList ([#2214](https://github.com/bluealloy/revm/pull/2214))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.4...revm-context-v1.0.0-alpha.5) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.3...revm-context-v1.0.0-alpha.4) - 2025-03-12

### Added

- add custom error to context ([#2197](https://github.com/bluealloy/revm/pull/2197))
- Add tx/block to EvmExecution trait ([#2195](https://github.com/bluealloy/revm/pull/2195))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.2...revm-context-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.1...revm-context-v1.0.0-alpha.2) - 2025-03-10

### Added

- added with_ref_db fn to Context ([#2164](https://github.com/bluealloy/revm/pull/2164))
- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))
- make journal entries generic ([#2154](https://github.com/bluealloy/revm/pull/2154))
- Standalone Host, remove default fn from context ([#2147](https://github.com/bluealloy/revm/pull/2147))
- add constructor with hardfork ([#2135](https://github.com/bluealloy/revm/pull/2135))
- allow host to be implemented on custom context ([#2112](https://github.com/bluealloy/revm/pull/2112))
- add the debug impl for Evm and EvmData type ([#2126](https://github.com/bluealloy/revm/pull/2126))

### Other

- pre EIP-7702 does not need to load code ([#2162](https://github.com/bluealloy/revm/pull/2162))
- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- rename transact_previous to replay, move EvmTr traits ([#2153](https://github.com/bluealloy/revm/pull/2153))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/bluealloy/revm/pull/2132))
- Adding function derive_tx_type to TxEnv ([#2118](https://github.com/bluealloy/revm/pull/2118))
- remove wrong `&mut`/`TODO`, and avoid useless `get_mut` ([#2111](https://github.com/bluealloy/revm/pull/2111))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-context-v1.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- Introduce Auth and AccessList traits (#2079)
- integrate alloy-eips (#2078)
- *(eip7702)* devnet6 changes and bump eest tests (#2055)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- simplify Transaction trait (#1959)
- align Block trait (#1957)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- clear JournalState and set first journal vec (#1929)
- Clear journal (#1927)
- *(revme)* include correct bytecode for snailtracer  (#1917)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Add helpers with_inspector with_precompile (#2063)
- simplify some generics (#2032)
- Add helper functions for JournalInit #1879 (#1961)
- fix journal naming for inc/dec balance (#1976)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- tie journal database with database getter (#1923)
- Move CfgEnv from context-interface to context crate (#1910)
- Bump new logo (#1735)
- *(README)* add rbuilder to used-by (#1585)
- added simular to used-by (#1521)
- add Trin to used by list (#1393)
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))
- Add Hardhat to the "Used by" list ([#1164](https://github.com/bluealloy/revm/pull/1164))
- Add VERBS to used by list ([#1141](https://github.com/bluealloy/revm/pull/1141))
- license date and revm docs (#1080)
- *(docs)* Update the benchmark docs to point to revm package (#906)
- *(docs)* Update top-level benchmark docs (#894)
- clang requirement (#784)
- Readme Updates (#756)
- Logo (#743)
- book workflow ([#537](https://github.com/bluealloy/revm/pull/537))
- add example to revm crate ([#468](https://github.com/bluealloy/revm/pull/468))
- Update README.md ([#424](https://github.com/bluealloy/revm/pull/424))
- add no_std to primitives ([#366](https://github.com/bluealloy/revm/pull/366))
- revm-precompiles to revm-precompile
- Bump v20, changelog ([#350](https://github.com/bluealloy/revm/pull/350))
- typos (#232)
- Add support for old forks. ([#191](https://github.com/bluealloy/revm/pull/191))
- revm bump 1.8. update libs. snailtracer rename ([#159](https://github.com/bluealloy/revm/pull/159))
- typo fixes
- fix readme typo
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/bluealloy/revm/pull/52))
- readme. debuger update
- Bump revm v0.3.0. README updated
- readme
- Add time elapsed for tests
- readme updated
- Include Basefee into cost calc. readme change
- Initialize precompile accounts
- Status update. Taking a break
- Merkle calc. Tweaks and debugging for eip158
- Replace aurora bn lib with parity's. All Bn128Add/Mul/Pair tests passes
- TEMP
- one tab removed
- readme
- README Example simplified
- Gas calculation for Call/Create. Example Added
- readme usage
- README changes
- Static gas cost added
- Subroutine changelogs and reverts
- Readme postulates
- Spelling
- Restructure project
- First iteration. Machine is looking okay
