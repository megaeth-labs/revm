# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [19.1.0](https://github.com/megaeth-labs/revm/compare/revm-statetest-types-v19.0.3...revm-statetest-types-v19.1.0) - 2026-09-08

### Added

- *(statetest-types)* add slot_number support (EIP-7843) ([#3391](https://github.com/megaeth-labs/revm/pull/3391))
- Implement EIP-7843 SLOTNUM opcode for Amsterdam ([#3340](https://github.com/megaeth-labs/revm/pull/3340))
- move GasParams to Cfg ([#3229](https://github.com/megaeth-labs/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/megaeth-labs/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/megaeth-labs/revm/pull/3070))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/megaeth-labs/revm/pull/2935))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/megaeth-labs/revm/pull/2377))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- Introduce Auth and AccessList traits ([#2079](https://github.com/megaeth-labs/revm/pull/2079))
- integrate alloy-eips ([#2078](https://github.com/megaeth-labs/revm/pull/2078))
- *(EIP-7623)* adjuct floor gas check order (main) ([#1991](https://github.com/megaeth-labs/revm/pull/1991))
- *(EIP-7840)* Add blob schedule to execution client cfg ([#1980](https://github.com/megaeth-labs/revm/pull/1980))
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 ([#1969](https://github.com/megaeth-labs/revm/pull/1969))
- simplify Transaction trait ([#1959](https://github.com/megaeth-labs/revm/pull/1959))
- Restucturing Part7 Handler and Context rework ([#1865](https://github.com/megaeth-labs/revm/pull/1865))
- restructuring Part6 transaction crate ([#1814](https://github.com/megaeth-labs/revm/pull/1814))
- extract statetest models/structs to standalone crate ([#1808](https://github.com/megaeth-labs/revm/pull/1808))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))

### Fixed

- *(statetest)* use spec-aware blob base fee update fraction ([#3210](https://github.com/megaeth-labs/revm/pull/3210))
- *(revme)* use primitive hashmap in statetest ([#3137](https://github.com/megaeth-labs/revm/pull/3137))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- *(eof)* dont run precompile on ext delegate call ([#1964](https://github.com/megaeth-labs/revm/pull/1964))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))

### Other

- release prep — bump all crates with unpublished changes ([#3721](https://github.com/megaeth-labs/revm/pull/3721))
- release ([#3705](https://github.com/megaeth-labs/revm/pull/3705))
- release ([#3701](https://github.com/megaeth-labs/revm/pull/3701))
- v109 release prep ([#3695](https://github.com/megaeth-labs/revm/pull/3695))
- v108 release prep ([#3689](https://github.com/megaeth-labs/revm/pull/3689))
- release ([#3679](https://github.com/megaeth-labs/revm/pull/3679))
- backport v107 release notes from branch ([#3617](https://github.com/megaeth-labs/revm/pull/3617))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/megaeth-labs/revm/pull/3568))
- v106 release prep ([#3554](https://github.com/megaeth-labs/revm/pull/3554))
- release ([#3472](https://github.com/megaeth-labs/revm/pull/3472))
- remove unused bytecode dependency from revm-statetest-types ([#3485](https://github.com/megaeth-labs/revm/pull/3485))
- bump revm-database-interface to v10.0.0 and all dependents (v107) ([#3474](https://github.com/megaeth-labs/revm/pull/3474))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- move EIP-161 state clear into journal finalize ([#3444](https://github.com/megaeth-labs/revm/pull/3444))
- *(statetest-types)* add BPO2ToAmsterdamAtTime15k fork spec ([#3397](https://github.com/megaeth-labs/revm/pull/3397))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/megaeth-labs/revm/pull/3358))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
- apply improvements from ai-bot labeled PRs ([#3297](https://github.com/megaeth-labs/revm/pull/3297))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/megaeth-labs/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/megaeth-labs/revm/pull/3272))
- re-export statetest-types from revm crate behind test-types feature ([#3247](https://github.com/megaeth-labs/revm/pull/3247))
- *(fmt)* merge all imports ([#3184](https://github.com/megaeth-labs/revm/pull/3184))
- tag v102 revm v33.1.0 ([#3177](https://github.com/megaeth-labs/revm/pull/3177))
- release ([#3162](https://github.com/megaeth-labs/revm/pull/3162))
- tag v100 revm v33.0.0 ([#3161](https://github.com/megaeth-labs/revm/pull/3161))
- v99 revm v32.0.0 ([#3157](https://github.com/megaeth-labs/revm/pull/3157))
- release ([#3136](https://github.com/megaeth-labs/revm/pull/3136))
- merge v98 versions bumps ([#3155](https://github.com/megaeth-labs/revm/pull/3155))
- tag v96 revm v31.0.0 ([#3135](https://github.com/megaeth-labs/revm/pull/3135))
- release ([#3113](https://github.com/megaeth-labs/revm/pull/3113))
- tag v93 revm v30.1.0 ([#3112](https://github.com/megaeth-labs/revm/pull/3112))
- release ([#3108](https://github.com/megaeth-labs/revm/pull/3108))
- release ([#3102](https://github.com/megaeth-labs/revm/pull/3102))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- bump minor versions ([#3078](https://github.com/megaeth-labs/revm/pull/3078))
- release ([#3061](https://github.com/megaeth-labs/revm/pull/3061))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- changelog update for v87 ([#3056](https://github.com/megaeth-labs/revm/pull/3056))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- *(btest)* 0 chainid is considered none ([#3022](https://github.com/megaeth-labs/revm/pull/3022))
- prealloc few frames ([#2965](https://github.com/megaeth-labs/revm/pull/2965))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- remove parent blob gas used and excess ([#2933](https://github.com/megaeth-labs/revm/pull/2933))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/megaeth-labs/revm/pull/2925))
- release ([#2899](https://github.com/megaeth-labs/revm/pull/2899))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- tag v84 revm v28.0.0 ([#2856](https://github.com/megaeth-labs/revm/pull/2856))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- release ([#2682](https://github.com/megaeth-labs/revm/pull/2682))
- tag v81 revm v27.0.1 ([#2689](https://github.com/megaeth-labs/revm/pull/2689))
- release ([#2659](https://github.com/megaeth-labs/revm/pull/2659))
- cargo clippy --fix --all ([#2671](https://github.com/megaeth-labs/revm/pull/2671))
- statetest runner cleanup ([#2665](https://github.com/megaeth-labs/revm/pull/2665))
- release ([#2657](https://github.com/megaeth-labs/revm/pull/2657))
- bump v77 ([#2651](https://github.com/megaeth-labs/revm/pull/2651))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- lints for examples ([#2650](https://github.com/megaeth-labs/revm/pull/2650))
- tag v76 revm v25.0.0 ([#2590](https://github.com/megaeth-labs/revm/pull/2590))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- tag v75 revm v24.0.1 ([#2563](https://github.com/megaeth-labs/revm/pull/2563)) ([#2589](https://github.com/megaeth-labs/revm/pull/2589))
- unify calling of journal account loading ([#2561](https://github.com/megaeth-labs/revm/pull/2561))
- tag v74 revm v24.0.0 ([#2539](https://github.com/megaeth-labs/revm/pull/2539))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- add TxEnvBuilder::build_fill ([#2536](https://github.com/megaeth-labs/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/megaeth-labs/revm/pull/2461))
- tag v71, revm v23.1.0 semver major bump ([#2492](https://github.com/megaeth-labs/revm/pull/2492))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/megaeth-labs/revm/pull/2415)) ([#2416](https://github.com/megaeth-labs/revm/pull/2416))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- tag v67 revm v21.0.0 ([#2341](https://github.com/megaeth-labs/revm/pull/2341))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- revm-statetest-types to alpha5 ([#2220](https://github.com/megaeth-labs/revm/pull/2220))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- tag v60, revm v20.0.0-alpha.3 ([#2176](https://github.com/megaeth-labs/revm/pull/2176))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/megaeth-labs/revm/pull/2151))
- allow duplicate v and yparity in test files ([#2150](https://github.com/megaeth-labs/revm/pull/2150))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/megaeth-labs/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- Bump licence year to 2025 ([#2058](https://github.com/megaeth-labs/revm/pull/2058))
- bump devnet5 v1.3.0 tests ([#2020](https://github.com/megaeth-labs/revm/pull/2020))
- align crates versions ([#1983](https://github.com/megaeth-labs/revm/pull/1983))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
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

## [19.0.3](https://github.com/bluealloy/revm/compare/revm-statetest-types-v19.0.2...revm-statetest-types-v19.0.3) - 2026-05-26

### Other

- updated the following local packages: revm-database, revm-context

## [19.0.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v19.0.1...revm-statetest-types-v19.0.2) - 2026-05-22

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-database

## [19.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v19.0.0...revm-statetest-types-v19.0.1) - 2026-05-21

### Other

- updated the following local packages: revm-context-interface, revm-context

## [19.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v17.0.1...revm-statetest-types-v19.0.0) - 2026-05-19

### Other

- backport v107 release notes from branch ([#3617](https://github.com/bluealloy/revm/pull/3617))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [17.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v17.0.0...revm-statetest-types-v17.0.1) - 2026-04-17

### Other

- updated the following local packages: revm-state, revm-context-interface, revm-context, revm-database

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v16.0.0...revm-statetest-types-v17.0.0) - 2026-04-10

### Other

- remove unused bytecode dependency from revm-statetest-types ([#3485](https://github.com/bluealloy/revm/pull/3485))

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v15.0.0...revm-statetest-types-v16.0.0) - 2026-03-04

### Other

- bump revm-database-interface to v10.0.0

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v14.0.0...revm-statetest-types-v15.0.0) - 2026-03-02

### Added

- *(statetest-types)* add slot_number support (EIP-7843) ([#3391](https://github.com/bluealloy/revm/pull/3391))
- Implement EIP-7843 SLOTNUM opcode for Amsterdam ([#3340](https://github.com/bluealloy/revm/pull/3340))

### Other

- move EIP-161 state clear into journal finalize ([#3444](https://github.com/bluealloy/revm/pull/3444))
- *(statetest-types)* add BPO2ToAmsterdamAtTime15k fork spec ([#3397](https://github.com/bluealloy/revm/pull/3397))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v13.1.0...revm-statetest-types-v14.0.0) - 2026-01-15

### Added

- move GasParams to Cfg ([#3229](https://github.com/bluealloy/revm/pull/3229))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/bluealloy/revm/pull/3252))
- BAL EIP-7928 ([#3070](https://github.com/bluealloy/revm/pull/3070))

### Fixed

- *(statetest)* use spec-aware blob base fee update fraction ([#3210](https://github.com/bluealloy/revm/pull/3210))

### Other

- apply improvements from ai-bot labeled PRs ([#3297](https://github.com/bluealloy/revm/pull/3297))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- re-export statetest-types from revm crate behind test-types feature ([#3247](https://github.com/bluealloy/revm/pull/3247))
- *(fmt)* merge all imports ([#3184](https://github.com/bluealloy/revm/pull/3184))

## [13.1.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v13.0.0...revm-statetest-types-v13.1.0) - 2025-11-14

### Other

- updated the following local packages: revm

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v11.0.2...revm-statetest-types-v13.0.0) - 2025-11-10

### Other

- updated the following local packages: revm

## [11.0.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v11.0.1...revm-statetest-types-v11.0.2) - 2025-11-10

### Other

- updated the following local packages: revm

## [11.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v11.0.0...revm-statetest-types-v11.0.1) - 2025-11-07

### Fixed

- *(revme)* use primitive hashmap in statetest ([#3137](https://github.com/bluealloy/revm/pull/3137))

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v10.2.0...revm-statetest-types-v11.0.0) - 2025-10-30

### Other

- updated the following local packages: revm

## [10.2.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v10.1.2...revm-statetest-types-v10.2.0) - 2025-10-17

### Other

- updated the following local packages: revm

## [10.1.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v10.1.1...revm-statetest-types-v10.1.2) - 2025-10-15

### Other

- updated the following local packages: revm

## [10.1.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v10.1.0...revm-statetest-types-v10.1.1) - 2025-10-15

### Other

- updated the following local packages: revm

## [10.1.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v10.0.0...revm-statetest-types-v10.1.0) - 2025-10-09

### Other

- updated the following local packages: revm

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v9.0.3...revm-statetest-types-v10.0.0) - 2025-10-07

### Added

- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- *(btest)* 0 chainid is considered none ([#3022](https://github.com/bluealloy/revm/pull/3022))
- prealloc few frames ([#2965](https://github.com/bluealloy/revm/pull/2965))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- remove parent blob gas used and excess ([#2933](https://github.com/bluealloy/revm/pull/2933))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/bluealloy/revm/pull/2925))

## [9.0.3](https://github.com/bluealloy/revm/compare/revm-statetest-types-v9.0.2...revm-statetest-types-v9.0.3) - 2025-09-23

### Other

- updated the following local packages: revm

## [9.0.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v9.0.1...revm-statetest-types-v9.0.2) - 2025-08-23

### Other

- updated the following local packages: revm

## [9.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v9.0.0...revm-statetest-types-v9.0.1) - 2025-08-12

### Other

- updated the following local packages: revm

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.5...revm-statetest-types-v9.0.0) - 2025-08-06

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))

## [8.0.5](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.4...revm-statetest-types-v8.0.5) - 2025-07-23

### Other

- updated the following local packages: revm

## [8.0.4](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.3...revm-statetest-types-v8.0.4) - 2025-07-14

### Other

- updated the following local packages: revm

## [8.0.3](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.2...revm-statetest-types-v8.0.3) - 2025-07-03

### Other

- updated the following local packages: revm

## [8.0.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.1...revm-statetest-types-v8.0.2) - 2025-06-30

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))
- statetest runner cleanup ([#2665](https://github.com/bluealloy/revm/pull/2665))

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v8.0.0...revm-statetest-types-v8.0.1) - 2025-06-20

### Other

- updated the following local packages: revm

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v7.0.0...revm-statetest-types-v8.0.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))

### Other

- lints for examples ([#2650](https://github.com/bluealloy/revm/pull/2650))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v6.0.0...revm-statetest-types-v7.0.0) - 2025-06-06

### Added

- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))
- unify calling of journal account loading ([#2561](https://github.com/bluealloy/revm/pull/2561))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v5.0.0...revm-statetest-types-v6.0.0) - 2025-05-31

### Other

- unify calling of journal account loading

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v4.1.0...revm-statetest-types-v5.0.0) - 2025-05-22

### Other

- add TxEnvBuilder::build_fill ([#2536](https://github.com/bluealloy/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [4.1.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v4.0.0...revm-statetest-types-v4.1.0) - 2025-05-07

Dependency bump

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v3.0.1...revm-statetest-types-v4.0.0) - 2025-05-07

### Added

- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v3.0.0..revm-statetest-types-v3.0.1) - 2025-04-15

### Other

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v2.0.0...revm-statetest-types-v3.0.0) - 2025-04-09

### Other

- updated the following local packages: revm

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0...revm-statetest-types-v2.0.0) - 2025-03-28

### Other

- updated the following local packages: revm

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.4...revm-statetest-types-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.4...revm-statetest-types-v1.0.0-alpha.5) - 2025-03-11

### Other

- updated the following local packages: revm

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.3...revm-statetest-types-v1.0.0-alpha.4) - 2025-03-11

### Other

- updated the following local packages: revm

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.2...revm-statetest-types-v1.0.0-alpha.3) - 2025-03-10

### Other

- updated the following local packages: revm

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.1...revm-statetest-types-v1.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- allow duplicate v and yparity in test files ([#2150](https://github.com/bluealloy/revm/pull/2150))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-statetest-types-v1.0.0-alpha.1) - 2025-02-16

### Added

- Introduce Auth and AccessList traits (#2079)
- integrate alloy-eips (#2078)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- simplify Transaction trait (#1959)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- extract statetest models/structs to standalone crate (#1808)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- *(eof)* dont run precompile on ext delegate call (#1964)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- bump devnet5 v1.3.0 tests (#2020)
- align crates versions (#1983)
- fix comments and docs into more sensible (#1920)
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
