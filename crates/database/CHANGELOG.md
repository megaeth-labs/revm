# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [15.1.0](https://github.com/megaeth-labs/revm/compare/revm-database-v15.0.2...revm-database-v15.1.0) - 2026-09-08

### Added

- add `OnStateHook` for `State<DB>` ([#3710](https://github.com/megaeth-labs/revm/pull/3710))
- *(database)* add is_fatal to DBErrorMarker ([#3704](https://github.com/megaeth-labs/revm/pull/3704))
- show address or slot in BAL error ([#3619](https://github.com/megaeth-labs/revm/pull/3619))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/megaeth-labs/revm/pull/3610))
- *(account)* Optimized index type for account ID using `non-max` ([#3605](https://github.com/megaeth-labs/revm/pull/3605))
- *(database)* add State::has_bal helper ([#3604](https://github.com/megaeth-labs/revm/pull/3604))
- add crate-level re-exports for all revm-* dependencies ([#3507](https://github.com/megaeth-labs/revm/pull/3507))
- add extend method to BlockHashCache ([#3471](https://github.com/megaeth-labs/revm/pull/3471))
- *(database)* add clear() to CacheState and TransitionState ([#3390](https://github.com/megaeth-labs/revm/pull/3390))
- *(database)* add iter and lowest helpers to BlockHashCache ([#3352](https://github.com/megaeth-labs/revm/pull/3352))
- *(database)* add StateBuilder::with_bal_builder_if ([#3351](https://github.com/megaeth-labs/revm/pull/3351))
- *(database)* add storage getter to BundleState ([#3321](https://github.com/megaeth-labs/revm/pull/3321))
- *(cache-db)* Added pritty_print for CacheDB ([#3296](https://github.com/megaeth-labs/revm/pull/3296))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/megaeth-labs/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/megaeth-labs/revm/pull/3070))
- DatabaseCommitExt::drain_balances ([#3205](https://github.com/megaeth-labs/revm/pull/3205))
- DatabaseCommitExt + increment_balances ([#3195](https://github.com/megaeth-labs/revm/pull/3195))
- DatabaseCommit::commit_iter ([#3197](https://github.com/megaeth-labs/revm/pull/3197))
- Restrict Database::Error. JournaledAccountTr ([#3199](https://github.com/megaeth-labs/revm/pull/3199))
- implement Database traits for either::Either ([#2673](https://github.com/megaeth-labs/revm/pull/2673))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- *(database)* Implement DatabaseRef for State ([#2570](https://github.com/megaeth-labs/revm/pull/2570))
- added TxEnv::new_bench() add util function ([#2556](https://github.com/megaeth-labs/revm/pull/2556))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- *(database)* re-export all public items from alloydb when feature … ([#2443](https://github.com/megaeth-labs/revm/pull/2443))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- TryDatabaseCommit ([#2121](https://github.com/megaeth-labs/revm/pull/2121))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- Evm structure (Cached Instructions and Precompiles) ([#2049](https://github.com/megaeth-labs/revm/pull/2049))
- Context execution ([#2013](https://github.com/megaeth-labs/revm/pull/2013))
- EthHandler trait ([#2001](https://github.com/megaeth-labs/revm/pull/2001))
- expose precompile address in Journal, DB::Error: StdError ([#1956](https://github.com/megaeth-labs/revm/pull/1956))
- integrate codspeed ([#1935](https://github.com/megaeth-labs/revm/pull/1935))
- *(database)* implement order-independent equality for Reverts ([#1827](https://github.com/megaeth-labs/revm/pull/1827))
- couple convenience functions for nested cache dbs ([#1852](https://github.com/megaeth-labs/revm/pull/1852))
- Restucturing Part7 Handler and Context rework ([#1865](https://github.com/megaeth-labs/revm/pull/1865))
- add support for async database ([#1809](https://github.com/megaeth-labs/revm/pull/1809))
- restructure part3 fix examples  ([#1792](https://github.com/megaeth-labs/revm/pull/1792))
- restructure Part2 database crate ([#1784](https://github.com/megaeth-labs/revm/pull/1784))
- project restructuring Part1 ([#1776](https://github.com/megaeth-labs/revm/pull/1776))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))

### Fixed

- gracefully handle commits of non-cached accounts ([#3657](https://github.com/megaeth-labs/revm/pull/3657))
- forward storage_by_account_id in WrapDatabaseRef ([#3441](https://github.com/megaeth-labs/revm/pull/3441))
- *(database)* allow EIP161 state clear for empty Loaded and Changed accounts ([#3421](https://github.com/megaeth-labs/revm/pull/3421))
- *(db)* use self.storage() in storage_by_account_id to avoid cache bypass ([#3385](https://github.com/megaeth-labs/revm/pull/3385))
- *(bal)* fix populated account if pre-state was none ([#3357](https://github.com/megaeth-labs/revm/pull/3357))
- *(database)* BlockHashCache incorrectly returns zero for block 0 ([#3319](https://github.com/megaeth-labs/revm/pull/3319))
- *(database)* return error instead of panic when block not found in AlloyDB ([#3284](https://github.com/megaeth-labs/revm/pull/3284))
- *(database)* make DatabaseCommit dyn-compatible ([#3264](https://github.com/megaeth-labs/revm/pull/3264))
- *(database)* prevent deadlock in ([#3251](https://github.com/megaeth-labs/revm/pull/3251))
- *(database)* verify handle belongs to current runtime before block_in_place ([#3212](https://github.com/megaeth-labs/revm/pull/3212))
- *(database)* return correct bytecode in BenchmarkDB::code_by_hash ([#3170](https://github.com/megaeth-labs/revm/pull/3170))
- *(op)* Ensure L1Block account is always loaded ([#3150](https://github.com/megaeth-labs/revm/pull/3150)) ([#3154](https://github.com/megaeth-labs/revm/pull/3154))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- change account state to None if NotExisting on insert_account_info ([#2630](https://github.com/megaeth-labs/revm/pull/2630))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- remove wrong Clone Macro in WrapDatabaseRef ([#2181](https://github.com/megaeth-labs/revm/pull/2181))
- correct propagate features ([#2177](https://github.com/megaeth-labs/revm/pull/2177))
- use correct HashMap import ([#2148](https://github.com/megaeth-labs/revm/pull/2148))
- *(op)* Handler deposit tx halt, catch_error handle ([#2144](https://github.com/megaeth-labs/revm/pull/2144))
- no_std for revm-database ([#2077](https://github.com/megaeth-labs/revm/pull/2077))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))

### Other

- release prep — bump all crates with unpublished changes ([#3721](https://github.com/megaeth-labs/revm/pull/3721))
- release ([#3705](https://github.com/megaeth-labs/revm/pull/3705))
- v109 release prep ([#3695](https://github.com/megaeth-labs/revm/pull/3695))
- v108 release prep ([#3689](https://github.com/megaeth-labs/revm/pull/3689))
- release ([#3679](https://github.com/megaeth-labs/revm/pull/3679))
- *(database)* preserve commit_iter semantics ([#3681](https://github.com/megaeth-labs/revm/pull/3681))
- update alloy-eip7928 to newer version ([#3627](https://github.com/megaeth-labs/revm/pull/3627))
- backport v107 release notes from branch ([#3617](https://github.com/megaeth-labs/revm/pull/3617))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/megaeth-labs/revm/pull/3592))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/megaeth-labs/revm/pull/3568))
- v106 release prep ([#3554](https://github.com/megaeth-labs/revm/pull/3554))
- release ([#3472](https://github.com/megaeth-labs/revm/pull/3472))
- remove no-op background transition merge builder toggle ([#3510](https://github.com/megaeth-labs/revm/pull/3510))
- bump revm-database-interface to v10.0.0 and all dependents (v107) ([#3474](https://github.com/megaeth-labs/revm/pull/3474))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- move EIP-161 state clear into journal finalize ([#3444](https://github.com/megaeth-labs/revm/pull/3444))
- deduplicate CacheDB::basic by delegating to load_account ([#3447](https://github.com/megaeth-labs/revm/pull/3447))
- *(database)* add reserve calls in merge_transitions and extend_state ([#3430](https://github.com/megaeth-labs/revm/pull/3430))
- reserve capacity in add_transitions ([#3373](https://github.com/megaeth-labs/revm/pull/3373))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/megaeth-labs/revm/pull/3358))
- Use O(1) ring buffer cache for block hashes instead of BTreeMap ([#3299](https://github.com/megaeth-labs/revm/pull/3299))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/megaeth-labs/revm/pull/3294))
- fix typos and grammar in database crate ([#3279](https://github.com/megaeth-labs/revm/pull/3279))
- happy new year, 2026 licence ([#3272](https://github.com/megaeth-labs/revm/pull/3272))
- avoid collect in CacheState commit ([#3242](https://github.com/megaeth-labs/revm/pull/3242))
- *(database)* use fixed hashmaps in cache db ([#3231](https://github.com/megaeth-labs/revm/pull/3231))
- *(database)* avoid triple cache lookup ([#3232](https://github.com/megaeth-labs/revm/pull/3232))
- optimize vector initialization with size hints in state and precompile modules ([#3191](https://github.com/megaeth-labs/revm/pull/3191))
- *(fmt)* merge all imports ([#3184](https://github.com/megaeth-labs/revm/pull/3184))
- release ([#3162](https://github.com/megaeth-labs/revm/pull/3162))
- merge v98 versions bumps ([#3155](https://github.com/megaeth-labs/revm/pull/3155))
- release ([#3113](https://github.com/megaeth-labs/revm/pull/3113))
- release ([#3102](https://github.com/megaeth-labs/revm/pull/3102))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- release ([#3061](https://github.com/megaeth-labs/revm/pull/3061))
- *(database)* optimize BTreeMap lookup in BundleState::build() ([#3068](https://github.com/megaeth-labs/revm/pull/3068))
- *(database)* remove unnecessary Send+Sync bounds from TryDatabaseCommit for Arc ([#3063](https://github.com/megaeth-labs/revm/pull/3063))
- remove deprecated methods ([#3050](https://github.com/megaeth-labs/revm/pull/3050))
- tag v88 revm v30.0.0 ([#3058](https://github.com/megaeth-labs/revm/pull/3058))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- *(database)* extract duplicate test balance constants ([#3017](https://github.com/megaeth-labs/revm/pull/3017))
- pretty print state in revme statetest ([#2979](https://github.com/megaeth-labs/revm/pull/2979))
- *(database)* avoid panic by conditionally using block_in_place ([#2927](https://github.com/megaeth-labs/revm/pull/2927))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- release ([#2899](https://github.com/megaeth-labs/revm/pull/2899))
- *(database)* remove unused dependencies ([#2885](https://github.com/megaeth-labs/revm/pull/2885))
- add AccountStatus unit test ([#2869](https://github.com/megaeth-labs/revm/pull/2869))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- use mem::take ([#2870](https://github.com/megaeth-labs/revm/pull/2870))
- small performance and safety improvements ([#2868](https://github.com/megaeth-labs/revm/pull/2868))
- use HashMap::or_insert_with lazily compute ([#2864](https://github.com/megaeth-labs/revm/pull/2864))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- impl DatabaseRef for WrapDatabaseRef ([#2726](https://github.com/megaeth-labs/revm/pull/2726))
- tag v81 revm v27.0.1 ([#2689](https://github.com/megaeth-labs/revm/pull/2689))
- tag v79 revm v27.0.0 ([#2680](https://github.com/megaeth-labs/revm/pull/2680))
- release ([#2659](https://github.com/megaeth-labs/revm/pull/2659))
- fix copy-pasted inner doc comments ([#2663](https://github.com/megaeth-labs/revm/pull/2663))
- bump v77 ([#2651](https://github.com/megaeth-labs/revm/pull/2651))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- lints for revm-database ([#2639](https://github.com/megaeth-labs/revm/pull/2639))
- bump alloydb test ([#2640](https://github.com/megaeth-labs/revm/pull/2640))
- tag v76 revm v25.0.0 ([#2590](https://github.com/megaeth-labs/revm/pull/2590))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- Avoid clone before converting ref BundleAccount to CacheAccount ([#2574](https://github.com/megaeth-labs/revm/pull/2574))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/megaeth-labs/revm/pull/2568))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- bump alloy libs ([#2533](https://github.com/megaeth-labs/revm/pull/2533))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/megaeth-labs/revm/pull/2461))
- tag v71, revm v23.1.0 semver major bump ([#2492](https://github.com/megaeth-labs/revm/pull/2492))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- remove alloy-sol-types deps ([#2411](https://github.com/megaeth-labs/revm/pull/2411))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- clean unsed indicatif ([#2379](https://github.com/megaeth-labs/revm/pull/2379))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/megaeth-labs/revm/pull/2345))
- *(database)* remove auto_impl dependency ([#2344](https://github.com/megaeth-labs/revm/pull/2344))
- tag v67 revm v21.0.0 ([#2341](https://github.com/megaeth-labs/revm/pull/2341))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- Propagate asyncdb feature flag from database-interface to revm  ([#2310](https://github.com/megaeth-labs/revm/pull/2310))
- make number more readable ([#2300](https://github.com/megaeth-labs/revm/pull/2300))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- docs nits ([#2292](https://github.com/megaeth-labs/revm/pull/2292))
- v65 revm: v20.0.0-alpha.7 ([#2280](https://github.com/megaeth-labs/revm/pull/2280))
- make clippy happy ([#2274](https://github.com/megaeth-labs/revm/pull/2274))
- simplify single UT for OpSpecId compatibility. ([#2216](https://github.com/megaeth-labs/revm/pull/2216))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- bump alloy ([#2183](https://github.com/megaeth-labs/revm/pull/2183))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- *(db)* separate fields from `CacheDB` into `Cache` ([#2131](https://github.com/megaeth-labs/revm/pull/2131))
- PrecompileErrors to PrecompileError ([#2103](https://github.com/megaeth-labs/revm/pull/2103))
- *(deps)* bump breaking deps ([#2093](https://github.com/megaeth-labs/revm/pull/2093))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/megaeth-labs/revm/pull/2088))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- Rename NameTrait to NameTr ([#2084](https://github.com/megaeth-labs/revm/pull/2084))
- Bump licence year to 2025 ([#2058](https://github.com/megaeth-labs/revm/pull/2058))
- add comment for pub function and fix typo ([#2015](https://github.com/megaeth-labs/revm/pull/2015))
- bump alloy versions to match latest ([#2007](https://github.com/megaeth-labs/revm/pull/2007))
- Make inspector use generics, rm associated types ([#1934](https://github.com/megaeth-labs/revm/pull/1934))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
- bumps select alloy crates to 0.6 ([#1854](https://github.com/megaeth-labs/revm/pull/1854))
- Update README.md examples section ([#1853](https://github.com/megaeth-labs/revm/pull/1853))
- *(TransitionAccount)* remove unneeded clone ([#1860](https://github.com/megaeth-labs/revm/pull/1860))
- *(CacheAccount)* remove unneeded clone ([#1859](https://github.com/megaeth-labs/revm/pull/1859))
- bump alloy to 0.4.2 ([#1817](https://github.com/megaeth-labs/revm/pull/1817))
- *(primitives)* replace HashMap re-exports with alloy_primitives::map ([#1805](https://github.com/megaeth-labs/revm/pull/1805))
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

## [15.0.2](https://github.com/bluealloy/revm/compare/revm-database-v15.0.1...revm-database-v15.0.2) - 2026-05-26

### Added

- add `OnStateHook` for `State<DB>` ([#3710](https://github.com/bluealloy/revm/pull/3710))

## [15.0.1](https://github.com/bluealloy/revm/compare/revm-database-v15.0.0...revm-database-v15.0.1) - 2026-05-22

### Other

- updated the following local packages: revm-database-interface

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-database-v13.0.1...revm-database-v15.0.0) - 2026-05-19

### Added

- show address or slot in BAL error ([#3619](https://github.com/bluealloy/revm/pull/3619))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/bluealloy/revm/pull/3610))
- *(account)* Optimized index type for account ID using `non-max` ([#3605](https://github.com/bluealloy/revm/pull/3605))
- *(database)* add State::has_bal helper ([#3604](https://github.com/bluealloy/revm/pull/3604))

### Fixed

- gracefully handle commits of non-cached accounts ([#3657](https://github.com/bluealloy/revm/pull/3657))

### Other

- *(database)* preserve commit_iter semantics ([#3681](https://github.com/bluealloy/revm/pull/3681))
- update alloy-eip7928 to newer version ([#3627](https://github.com/bluealloy/revm/pull/3627))
- backport v107 release notes from branch ([#3617](https://github.com/bluealloy/revm/pull/3617))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/bluealloy/revm/pull/3592))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [13.0.1](https://github.com/bluealloy/revm/compare/revm-database-v13.0.0...revm-database-v13.0.1) - 2026-04-17

### Other

- updated the following local packages: revm-state, revm-database-interface

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-database-v12.0.0...revm-database-v13.0.0) - 2026-04-10

### Added

- add crate-level re-exports for all revm-* dependencies ([#3507](https://github.com/bluealloy/revm/pull/3507))

### Other

- remove no-op background transition merge builder toggle ([#3510](https://github.com/bluealloy/revm/pull/3510))

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-database-v11.0.0...revm-database-v12.0.0) - 2026-03-04

### Other

- bump revm-database-interface to v10.0.0

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-database-v10.0.0...revm-database-v11.0.0) - 2026-03-02

### Added

- *(database)* add clear() to CacheState and TransitionState ([#3390](https://github.com/bluealloy/revm/pull/3390))
- *(database)* add iter and lowest helpers to BlockHashCache ([#3352](https://github.com/bluealloy/revm/pull/3352))
- *(database)* add StateBuilder::with_bal_builder_if ([#3351](https://github.com/bluealloy/revm/pull/3351))
- *(database)* add storage getter to BundleState ([#3321](https://github.com/bluealloy/revm/pull/3321))

### Fixed

- forward storage_by_account_id in WrapDatabaseRef ([#3441](https://github.com/bluealloy/revm/pull/3441))
- *(database)* allow EIP161 state clear for empty Loaded and Changed accounts ([#3421](https://github.com/bluealloy/revm/pull/3421))
- *(db)* use self.storage() in storage_by_account_id to avoid cache bypass ([#3385](https://github.com/bluealloy/revm/pull/3385))
- *(bal)* fix populated account if pre-state was none ([#3357](https://github.com/bluealloy/revm/pull/3357))
- *(database)* BlockHashCache incorrectly returns zero for block 0 ([#3319](https://github.com/bluealloy/revm/pull/3319))

### Other

- move EIP-161 state clear into journal finalize ([#3444](https://github.com/bluealloy/revm/pull/3444))
- deduplicate CacheDB::basic by delegating to load_account ([#3447](https://github.com/bluealloy/revm/pull/3447))
- *(database)* add reserve calls in merge_transitions and extend_state ([#3430](https://github.com/bluealloy/revm/pull/3430))
- reserve capacity in add_transitions ([#3373](https://github.com/bluealloy/revm/pull/3373))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))
- Use O(1) ring buffer cache for block hashes instead of BTreeMap ([#3299](https://github.com/bluealloy/revm/pull/3299))

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-database-v9.0.6...revm-database-v10.0.0) - 2026-01-15

### Added

- *(cache-db)* Added pritty_print for CacheDB ([#3296](https://github.com/bluealloy/revm/pull/3296))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/bluealloy/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/bluealloy/revm/pull/3070))
- DatabaseCommitExt::drain_balances ([#3205](https://github.com/bluealloy/revm/pull/3205))
- DatabaseCommitExt + increment_balances ([#3195](https://github.com/bluealloy/revm/pull/3195))
- DatabaseCommit::commit_iter ([#3197](https://github.com/bluealloy/revm/pull/3197))
- Restrict Database::Error. JournaledAccountTr ([#3199](https://github.com/bluealloy/revm/pull/3199))

### Fixed

- *(database)* return error instead of panic when block not found in AlloyDB ([#3284](https://github.com/bluealloy/revm/pull/3284))
- *(database)* make DatabaseCommit dyn-compatible ([#3264](https://github.com/bluealloy/revm/pull/3264))
- *(database)* prevent deadlock in ([#3251](https://github.com/bluealloy/revm/pull/3251))
- *(database)* verify handle belongs to current runtime before block_in_place ([#3212](https://github.com/bluealloy/revm/pull/3212))

### Other

- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- fix typos and grammar in database crate ([#3279](https://github.com/bluealloy/revm/pull/3279))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- avoid collect in CacheState commit ([#3242](https://github.com/bluealloy/revm/pull/3242))
- *(database)* use fixed hashmaps in cache db ([#3231](https://github.com/bluealloy/revm/pull/3231))
- *(database)* avoid triple cache lookup ([#3232](https://github.com/bluealloy/revm/pull/3232))
- optimize vector initialization with size hints in state and precompile modules ([#3191](https://github.com/bluealloy/revm/pull/3191))

## [9.0.6](https://github.com/bluealloy/revm/compare/revm-database-v9.0.5...revm-database-v9.0.6) - 2025-11-14

### Fixed

- *(database)* return correct bytecode in BenchmarkDB::code_by_hash ([#3170](https://github.com/bluealloy/revm/pull/3170))

## [9.0.5](https://github.com/bluealloy/revm/compare/revm-database-v9.0.4...revm-database-v9.0.5) - 2025-11-10

### Fixed

- *(op)* Ensure L1Block account is always loaded ([#3150](https://github.com/bluealloy/revm/pull/3150))

## [9.0.4](https://github.com/bluealloy/revm/compare/revm-database-v9.0.3...revm-database-v9.0.4) - 2025-11-07

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-database-interface

## [9.0.3](https://github.com/bluealloy/revm/compare/revm-database-v9.0.2...revm-database-v9.0.3) - 2025-10-30

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface

## [9.0.2](https://github.com/bluealloy/revm/compare/revm-database-v9.0.1...revm-database-v9.0.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface

## [9.0.1](https://github.com/bluealloy/revm/compare/revm-database-v9.0.0...revm-database-v9.0.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-database-interface

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-database-v8.0.0...revm-database-v9.0.0) - 2025-10-09

### Other

- *(database)* optimize BTreeMap lookup in BundleState::build() ([#3068](https://github.com/bluealloy/revm/pull/3068))
- *(database)* remove unnecessary Send+Sync bounds from TryDatabaseCommit for Arc ([#3063](https://github.com/bluealloy/revm/pull/3063))
- remove deprecated methods ([#3050](https://github.com/bluealloy/revm/pull/3050))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-database-v7.0.5...revm-database-v8.0.0) - 2025-10-07

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- *(database)* extract duplicate test balance constants ([#3017](https://github.com/bluealloy/revm/pull/3017))
- pretty print state in revme statetest ([#2979](https://github.com/bluealloy/revm/pull/2979))
- *(database)* avoid panic by conditionally using block_in_place ([#2927](https://github.com/bluealloy/revm/pull/2927))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))

## [7.0.5](https://github.com/bluealloy/revm/compare/revm-database-v7.0.4...revm-database-v7.0.5) - 2025-08-23

### Other

- *(database)* remove unused dependencies ([#2885](https://github.com/bluealloy/revm/pull/2885))
- add AccountStatus unit test ([#2869](https://github.com/bluealloy/revm/pull/2869))

## [7.0.4](https://github.com/bluealloy/revm/compare/revm-database-v7.0.3...revm-database-v7.0.4) - 2025-08-12

### Other

- use mem::take ([#2870](https://github.com/bluealloy/revm/pull/2870))
- small performance and safety improvements ([#2868](https://github.com/bluealloy/revm/pull/2868))
- use HashMap::or_insert_with lazily compute ([#2864](https://github.com/bluealloy/revm/pull/2864))

## [7.0.3](https://github.com/bluealloy/revm/compare/revm-database-v7.0.2...revm-database-v7.0.3) - 2025-08-06

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))

## [7.0.2](https://github.com/bluealloy/revm/compare/revm-database-v7.0.1...revm-database-v7.0.2) - 2025-07-23

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-database-interface, revm-state

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-database-v7.0.0...revm-database-v7.0.1) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-database-v6.0.0...revm-database-v7.0.0) - 2025-06-30

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-database-v5.0.0...revm-database-v6.0.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))

### Fixed

- change account state to None if NotExisting on insert_account_info ([#2630](https://github.com/bluealloy/revm/pull/2630))

### Other

- lints for revm-database ([#2639](https://github.com/bluealloy/revm/pull/2639))
- bump alloydb test ([#2640](https://github.com/bluealloy/revm/pull/2640))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-database-v4.0.1...revm-database-v5.0.0) - 2025-06-06

### Added

- *(database)* Implement DatabaseRef for State ([#2570](https://github.com/bluealloy/revm/pull/2570))
- added TxEnv::new_bench() add util function ([#2556](https://github.com/bluealloy/revm/pull/2556))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- Avoid clone before converting ref BundleAccount to CacheAccount ([#2574](https://github.com/bluealloy/revm/pull/2574))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/bluealloy/revm/pull/2568))

## [4.0.1](https://github.com/bluealloy/revm/compare/revm-database-v4.0.0...revm-database-v4.0.1) - 2025-05-22

### Other

- bump alloy libs ([#2533](https://github.com/bluealloy/revm/pull/2533))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))


## [4.0.0](https://github.com/bluealloy/revm/compare/revm-database-v3.1.0...revm-database-v4.0.0) - 2025-05-07

Dependency bump

## [3.1.0](https://github.com/bluealloy/revm/compare/revm-database-v3.0.0...revm-database-v3.1.0) - 2025-05-07

### Added

- *(database)* re-export all public items from alloydb when feature … ([#2443](https://github.com/bluealloy/revm/pull/2443))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- remove alloy-sol-types deps ([#2411](https://github.com/bluealloy/revm/pull/2411))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-database-v2.0.0...revm-database-v3.0.0) - 2025-04-09

### Other

- clean unsed indicatif ([#2379](https://github.com/bluealloy/revm/pull/2379))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))
- *(database)* remove auto_impl dependency ([#2344](https://github.com/bluealloy/revm/pull/2344))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0...revm-database-v2.0.0) - 2025-03-28

### Other

- make number more readable ([#2300](https://github.com/bluealloy/revm/pull/2300))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0-alpha.5...revm-database-v1.0.0) - 2025-03-24

### Other

- docs nits ([#2292](https://github.com/bluealloy/revm/pull/2292))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0-alpha.4...revm-database-v1.0.0-alpha.5) - 2025-03-21

### Other

- make clippy happy ([#2274](https://github.com/bluealloy/revm/pull/2274))
- simplify single UT for OpSpecId compatibility. ([#2216](https://github.com/bluealloy/revm/pull/2216))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0-alpha.3...revm-database-v1.0.0-alpha.4) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0-alpha.2...revm-database-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- bump alloy ([#2183](https://github.com/bluealloy/revm/pull/2183))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-database-v1.0.0-alpha.1...revm-database-v1.0.0-alpha.2) - 2025-03-10

### Fixed

- use correct HashMap import ([#2148](https://github.com/bluealloy/revm/pull/2148))
- *(op)* Handler deposit tx halt, catch_error handle ([#2144](https://github.com/bluealloy/revm/pull/2144))

### Other

- *(db)* separate fields from `CacheDB` into `Cache` ([#2131](https://github.com/bluealloy/revm/pull/2131))
- PrecompileErrors to PrecompileError ([#2103](https://github.com/bluealloy/revm/pull/2103))
- *(deps)* bump breaking deps ([#2093](https://github.com/bluealloy/revm/pull/2093))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-database-v1.0.0-alpha.1) - 2025-02-16

### Added

- Context execution (#2013)
- EthHandler trait (#2001)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- integrate codspeed (#1935)
- *(database)* implement order-independent equality for Reverts (#1827)
- couple convenience functions for nested cache dbs (#1852)
- Restucturing Part7 Handler and Context rework (#1865)
- add support for async database (#1809)
- restructure Part2 database crate (#1784)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- no_std for revm-database (#2077)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- add comment for pub function and fix typo (#2015)
- bump alloy versions to match latest (#2007)
- fix comments and docs into more sensible (#1920)
- bumps select alloy crates to 0.6 (#1854)
- *(TransitionAccount)* remove unneeded clone (#1860)
- *(CacheAccount)* remove unneeded clone (#1859)
- bump alloy to 0.4.2 (#1817)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
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
