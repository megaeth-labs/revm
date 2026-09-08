# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [13.0.0](https://github.com/megaeth-labs/revm/compare/revm-state-v12.0.1...revm-state-v13.0.0) - 2026-09-08

### Added

- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/megaeth-labs/revm/pull/3667))
- add borrowed alloy BAL conversion ([#3670](https://github.com/megaeth-labs/revm/pull/3670))
- *(state)* add Bal::try_from_alloy ([#3665](https://github.com/megaeth-labs/revm/pull/3665))
- show address or slot in BAL error ([#3619](https://github.com/megaeth-labs/revm/pull/3619))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/megaeth-labs/revm/pull/3610))
- *(account)* Optimized index type for account ID using `non-max` ([#3605](https://github.com/megaeth-labs/revm/pull/3605))
- move GasParams to Cfg ([#3229](https://github.com/megaeth-labs/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/megaeth-labs/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/megaeth-labs/revm/pull/3070))
- *(precompiles/jovian)* add jovian precompiles to revm ([#3128](https://github.com/megaeth-labs/revm/pull/3128)) ([#3134](https://github.com/megaeth-labs/revm/pull/3134))
- JournaledAccount, a nice way to update and track changes ([#3086](https://github.com/megaeth-labs/revm/pull/3086))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- Account helper builder functions ([#2355](https://github.com/megaeth-labs/revm/pull/2355))
- support for system calls ([#2350](https://github.com/megaeth-labs/revm/pull/2350))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- *(database)* implement order-independent equality for Reverts ([#1827](https://github.com/megaeth-labs/revm/pull/1827))
- Restucturing Part7 Handler and Context rework ([#1865](https://github.com/megaeth-labs/revm/pull/1865))
- restructuring Part6 transaction crate ([#1814](https://github.com/megaeth-labs/revm/pull/1814))
- restructure Part2 database crate ([#1784](https://github.com/megaeth-labs/revm/pull/1784))
- project restructuring Part1 ([#1776](https://github.com/megaeth-labs/revm/pull/1776))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))

### Fixed

- *(state)* canonicalize BAL alloy ordering ([#3618](https://github.com/megaeth-labs/revm/pull/3618))
- *(bal)* record storage writes to zero for selfdestructed accounts ([#3573](https://github.com/megaeth-labs/revm/pull/3573))
- set transition_id to account ([#3353](https://github.com/megaeth-labs/revm/pull/3353))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- skip cold load on oog ([#2903](https://github.com/megaeth-labs/revm/pull/2903))
- manally implementation PartialOrd and Ord for AccountInfo ([#2835](https://github.com/megaeth-labs/revm/pull/2835))
- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/megaeth-labs/revm/pull/2581))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- correct propagate features ([#2177](https://github.com/megaeth-labs/revm/pull/2177))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))

### Other

- release prep — bump all crates with unpublished changes ([#3721](https://github.com/megaeth-labs/revm/pull/3721))
- release ([#3679](https://github.com/megaeth-labs/revm/pull/3679))
- expand `BalError` documentation ([#3666](https://github.com/megaeth-labs/revm/pull/3666))
- update alloy-eip7928 to newer version ([#3627](https://github.com/megaeth-labs/revm/pull/3627))
- audit #[allow] attributes ([#3611](https://github.com/megaeth-labs/revm/pull/3611))
- backport v107 release notes from branch ([#3617](https://github.com/megaeth-labs/revm/pull/3617))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/megaeth-labs/revm/pull/3592))
- no alloc for empty accounts ([#3590](https://github.com/megaeth-labs/revm/pull/3590))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/megaeth-labs/revm/pull/3568))
- v106 release prep ([#3554](https://github.com/megaeth-labs/revm/pull/3554))
- release ([#3472](https://github.com/megaeth-labs/revm/pull/3472))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- [**breaking**] flatten Bytecode ([#3375](https://github.com/megaeth-labs/revm/pull/3375))
- *(state)* avoid unnecessary clone, prealloc, and fix typo ([#3387](https://github.com/megaeth-labs/revm/pull/3387))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/megaeth-labs/revm/pull/3358))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
- apply improvements from ai-bot labeled PRs ([#3297](https://github.com/megaeth-labs/revm/pull/3297))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/megaeth-labs/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/megaeth-labs/revm/pull/3272))
- deduplicate local/global flags setup ([#3190](https://github.com/megaeth-labs/revm/pull/3190))
- *(fmt)* merge all imports ([#3184](https://github.com/megaeth-labs/revm/pull/3184))
- merge v98 versions bumps ([#3155](https://github.com/megaeth-labs/revm/pull/3155))
- release ([#3113](https://github.com/megaeth-labs/revm/pull/3113))
- release ([#3102](https://github.com/megaeth-labs/revm/pull/3102))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- tag v88 revm v30.0.0 ([#3058](https://github.com/megaeth-labs/revm/pull/3058))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- helper caller_initial_modification added ([#3032](https://github.com/megaeth-labs/revm/pull/3032))
- *(state)* remove unnecessary core::hash::Hash import from lib.rs ([#2959](https://github.com/megaeth-labs/revm/pull/2959))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- use primitives::HashMap default ([#2916](https://github.com/megaeth-labs/revm/pull/2916))
- release ([#2899](https://github.com/megaeth-labs/revm/pull/2899))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- small performance and safety improvements ([#2868](https://github.com/megaeth-labs/revm/pull/2868))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- tag v81 revm v27.0.1 ([#2689](https://github.com/megaeth-labs/revm/pull/2689))
- tag v79 revm v27.0.0 ([#2680](https://github.com/megaeth-labs/revm/pull/2680))
- release ([#2659](https://github.com/megaeth-labs/revm/pull/2659))
- fix copy-pasted inner doc comments ([#2663](https://github.com/megaeth-labs/revm/pull/2663))
- bump v77 ([#2651](https://github.com/megaeth-labs/revm/pull/2651))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- unify calling of journal account loading ([#2561](https://github.com/megaeth-labs/revm/pull/2561))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/megaeth-labs/revm/pull/2461))
- tag v71, revm v23.1.0 semver major bump ([#2492](https://github.com/megaeth-labs/revm/pull/2492))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- cache and use JumpTable::default ([#2439](https://github.com/megaeth-labs/revm/pull/2439))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- add AccountInfo setters ([#2429](https://github.com/megaeth-labs/revm/pull/2429))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- *(lints)* revm-context lints ([#2404](https://github.com/megaeth-labs/revm/pull/2404))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- *(state)* Add AccountInfo builder util methods ([#2357](https://github.com/megaeth-labs/revm/pull/2357))
- tag v67 revm v21.0.0 ([#2341](https://github.com/megaeth-labs/revm/pull/2341))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- v65 revm: v20.0.0-alpha.7 ([#2280](https://github.com/megaeth-labs/revm/pull/2280))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/megaeth-labs/revm/pull/2151))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- Bump licence year to 2025 ([#2058](https://github.com/megaeth-labs/revm/pull/2058))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
- Update README.md examples section ([#1853](https://github.com/megaeth-labs/revm/pull/1853))
- inline more `AccountInfo` fns and add docs ([#1819](https://github.com/megaeth-labs/revm/pull/1819))
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

## [12.0.1](https://github.com/bluealloy/revm/compare/revm-state-v12.0.0...revm-state-v12.0.1) - 2026-05-26

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-state-v11.0.1...revm-state-v12.0.0) - 2026-05-19

### Added

- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/bluealloy/revm/pull/3667))
- add borrowed alloy BAL conversion ([#3670](https://github.com/bluealloy/revm/pull/3670))
- *(state)* add Bal::try_from_alloy ([#3665](https://github.com/bluealloy/revm/pull/3665))
- show address or slot in BAL error ([#3619](https://github.com/bluealloy/revm/pull/3619))
- *(state)* Optimized index type for transaction ID using non-max ([#3610](https://github.com/bluealloy/revm/pull/3610))
- *(account)* Optimized index type for account ID using `non-max` ([#3605](https://github.com/bluealloy/revm/pull/3605))

### Fixed

- *(state)* canonicalize BAL alloy ordering ([#3618](https://github.com/bluealloy/revm/pull/3618))
- *(bal)* record storage writes to zero for selfdestructed accounts ([#3573](https://github.com/bluealloy/revm/pull/3573))

### Other

- expand `BalError` documentation ([#3666](https://github.com/bluealloy/revm/pull/3666))
- update alloy-eip7928 to newer version ([#3627](https://github.com/bluealloy/revm/pull/3627))
- audit #[allow] attributes ([#3611](https://github.com/bluealloy/revm/pull/3611))
- backport v107 release notes from branch ([#3617](https://github.com/bluealloy/revm/pull/3617))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/bluealloy/revm/pull/3592))
- no alloc for empty accounts ([#3590](https://github.com/bluealloy/revm/pull/3590))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [11.0.1](https://github.com/bluealloy/revm/compare/revm-state-v11.0.0...revm-state-v11.0.1) - 2026-04-17

### Fixed

- *(bal)* record storage writes to zero for selfdestructed accounts ([#3573](https://github.com/bluealloy/revm/pull/3573))

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-state-v10.0.0...revm-state-v11.0.0) - 2026-04-10

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-state-v9.0.0...revm-state-v10.0.0) - 2026-03-02

### Fixed

- set transition_id to account ([#3353](https://github.com/bluealloy/revm/pull/3353))

### Other

- [**breaking**] flatten Bytecode ([#3375](https://github.com/bluealloy/revm/pull/3375))
- *(state)* avoid unnecessary clone, prealloc, and fix typo ([#3387](https://github.com/bluealloy/revm/pull/3387))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-state-v8.1.1...revm-state-v9.0.0) - 2026-01-15

### Added

- move GasParams to Cfg ([#3229](https://github.com/bluealloy/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/bluealloy/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/bluealloy/revm/pull/3070))

### Other

- apply improvements from ai-bot labeled PRs ([#3297](https://github.com/bluealloy/revm/pull/3297))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- deduplicate local/global flags setup ([#3190](https://github.com/bluealloy/revm/pull/3190))
- *(fmt)* merge all imports ([#3184](https://github.com/bluealloy/revm/pull/3184))

## [8.1.1](https://github.com/bluealloy/revm/compare/revm-state-v8.1.0...revm-state-v8.1.1) - 2025-11-07

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [8.1.0](https://github.com/bluealloy/revm/compare/revm-state-v8.0.2...revm-state-v8.1.0) - 2025-10-30

### Added

- JournaledAccount, a nice way to update and track changes ([#3086](https://github.com/bluealloy/revm/pull/3086))

## [8.0.2](https://github.com/bluealloy/revm/compare/revm-state-v8.0.1...revm-state-v8.0.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-state-v8.0.0...revm-state-v8.0.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [7.0.5](https://github.com/bluealloy/revm/compare/revm-state-v7.0.4...revm-state-v7.0.5) - 2025-08-23

### Other

- updated the following local packages: revm-bytecode

## [7.0.4](https://github.com/bluealloy/revm/compare/revm-state-v7.0.3...revm-state-v7.0.4) - 2025-08-12

### Other

- small performance and safety improvements ([#2868](https://github.com/bluealloy/revm/pull/2868))

## [7.0.3](https://github.com/bluealloy/revm/compare/revm-state-v7.0.2...revm-state-v7.0.3) - 2025-08-06

### Fixed

- manally implementation PartialOrd and Ord for AccountInfo ([#2835](https://github.com/bluealloy/revm/pull/2835))

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [7.0.6](https://github.com/bluealloy/revm/compare/revm-state-v7.0.5...revm-state-v7.0.6) - 2025-10-07

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))
- skip cold load on oog ([#2903](https://github.com/bluealloy/revm/pull/2903))

### Other

- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- helper caller_initial_modification added ([#3032](https://github.com/bluealloy/revm/pull/3032))
- *(state)* remove unnecessary core::hash::Hash import from lib.rs ([#2959](https://github.com/bluealloy/revm/pull/2959))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- use primitives::HashMap default ([#2916](https://github.com/bluealloy/revm/pull/2916))

## [7.0.2](https://github.com/bluealloy/revm/compare/revm-state-v7.0.1...revm-state-v7.0.2) - 2025-07-23

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-state-v7.0.0...revm-state-v7.0.1) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode

## [6.0.1](https://github.com/bluealloy/revm/compare/revm-state-v6.0.0...revm-state-v6.0.1) - 2025-06-30

### Other

- fix copy-pasted inner doc comments ([#2663](https://github.com/bluealloy/revm/pull/2663))

## [5.1.0](https://github.com/bluealloy/revm/compare/revm-state-v5.0.0...revm-state-v5.1.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-state-v4.0.1...revm-state-v5.0.0) - 2025-06-06

### Added

- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Fixed

- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/bluealloy/revm/pull/2581))

### Other

- unify calling of journal account loading ([#2561](https://github.com/bluealloy/revm/pull/2561))

## [4.0.1](https://github.com/bluealloy/revm/compare/revm-state-v4.0.0...revm-state-v4.0.1) - 2025-05-22

### Other

- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-state-v3.0.0...revm-state-v3.0.1) - 2025-05-07

Yanked release

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- cache and use JumpTable::default ([#2439](https://github.com/bluealloy/revm/pull/2439))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- add AccountInfo setters ([#2429](https://github.com/bluealloy/revm/pull/2429))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-state-v2.0.0...revm-state-v3.0.0) - 2025-04-09

### Added

- Account helper builder functions ([#2355](https://github.com/bluealloy/revm/pull/2355))
- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))

### Other

- *(state)* Add AccountInfo builder util methods ([#2357](https://github.com/bluealloy/revm/pull/2357))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0...revm-state-v2.0.0) - 2025-03-28

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0-alpha.4...revm-state-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0-alpha.4...revm-state-v1.0.0-alpha.5) - 2025-03-21

### Other

- updated the following local packages: revm-primitives

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0-alpha.3...revm-state-v1.0.0-alpha.4) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0-alpha.2...revm-state-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-state-v1.0.0-alpha.1...revm-state-v1.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-state-v1.0.0-alpha.1) - 2025-02-16

### Added

- *(database)* implement order-independent equality for Reverts (#1827)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- fix comments and docs into more sensible (#1920)
- inline more `AccountInfo` fns and add docs (#1819)
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
