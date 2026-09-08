# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [24.1.0](https://github.com/megaeth-labs/revm/compare/revm-primitives-v24.0.1...revm-primitives-v24.1.0) - 2026-09-08

### Added

- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/megaeth-labs/revm/pull/3667))
- Part of amsterdam devnet3 EIP updates ([#3438](https://github.com/megaeth-labs/revm/pull/3438))
- Implement EIP-7708 ETH transfers emit a log ([#3334](https://github.com/megaeth-labs/revm/pull/3334))
- BAL EIP-7928 ([#3070](https://github.com/megaeth-labs/revm/pull/3070))
- JournaledAccount sload/sstore ([#3201](https://github.com/megaeth-labs/revm/pull/3201))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/megaeth-labs/revm/pull/2935))
- short address for journal cold/warm check ([#2849](https://github.com/megaeth-labs/revm/pull/2849))
- *(osaka)* update EIP-7825 constant ([#2753](https://github.com/megaeth-labs/revm/pull/2753))
- expose sha3-keccak in revm and revm-primitives ([#2713](https://github.com/megaeth-labs/revm/pull/2713))
- remove EOF ([#2644](https://github.com/megaeth-labs/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/megaeth-labs/revm/pull/2611)) ([#2642](https://github.com/megaeth-labs/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/megaeth-labs/revm/pull/2575))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- *(Osaka)* modexp input limit and gas change, EIP-7823 and EIP-7883 ([#2531](https://github.com/megaeth-labs/revm/pull/2531))
- *(Osaka)* disable EOF ([#2480](https://github.com/megaeth-labs/revm/pull/2480))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/megaeth-labs/revm/pull/2377))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 ([#1969](https://github.com/megaeth-labs/revm/pull/1969))
- simplify Transaction trait ([#1959](https://github.com/megaeth-labs/revm/pull/1959))
- restructuring Part6 transaction crate ([#1814](https://github.com/megaeth-labs/revm/pull/1814))
- restructure Part2 database crate ([#1784](https://github.com/megaeth-labs/revm/pull/1784))
- project restructuring Part1 ([#1776](https://github.com/megaeth-labs/revm/pull/1776))
- to_plain_state ([#1778](https://github.com/megaeth-labs/revm/pull/1778))
- introducing EvmWiring, a chain-specific configuration ([#1672](https://github.com/megaeth-labs/revm/pull/1672))
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/megaeth-labs/revm/pull/1695))
- c-kzg bump, cleanup on kzgsetting ([#1719](https://github.com/megaeth-labs/revm/pull/1719))
- *(EOF)* add evmone test suite ([#1689](https://github.com/megaeth-labs/revm/pull/1689))
- check for typos in CI ([#1686](https://github.com/megaeth-labs/revm/pull/1686))
- *(EOF)* Add non-returning CALLF/JUMPF checks ([#1663](https://github.com/megaeth-labs/revm/pull/1663))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/megaeth-labs/revm/pull/1648))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/megaeth-labs/revm/pull/1607))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/megaeth-labs/revm/pull/1558))
- *(Precompiles)* Throw fatal error if c-kzg is disabled ([#1589](https://github.com/megaeth-labs/revm/pull/1589))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/megaeth-labs/revm/pull/1565))
- add helper function to mape EVMError's Database error variant ([#1567](https://github.com/megaeth-labs/revm/pull/1567))
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/megaeth-labs/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/megaeth-labs/revm/pull/1504))
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/megaeth-labs/revm/pull/1499))
- Persist reverted account and storage slot lookups in `JournaledState` ([#1437](https://github.com/megaeth-labs/revm/pull/1437))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/megaeth-labs/revm/pull/1467))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/megaeth-labs/revm/pull/1436))
- *(EOF)* Add CALLF/JUMPF stack checks ([#1417](https://github.com/megaeth-labs/revm/pull/1417))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/megaeth-labs/revm/pull/1415))
- Add bytes to Bytecode ([#1396](https://github.com/megaeth-labs/revm/pull/1396))
- implement EIP-2935 ([#1354](https://github.com/megaeth-labs/revm/pull/1354))
- add `Bytecode::original_bytecode_slice` to match `BytecodeLocked` ([#1286](https://github.com/megaeth-labs/revm/pull/1286))
- add flag to force hashbrown usage ([#1284](https://github.com/megaeth-labs/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/megaeth-labs/revm/pull/1143))
- pass rand feature to alloy_primitives ([#1276](https://github.com/megaeth-labs/revm/pull/1276))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/megaeth-labs/revm/pull/1243))
- implement Default for EnvWithHandlerCfg ([#1206](https://github.com/megaeth-labs/revm/pull/1206))
- optional nonce check ([#1195](https://github.com/megaeth-labs/revm/pull/1195))
- *(primitives)* derive Default for SpecId ([#1171](https://github.com/megaeth-labs/revm/pull/1171))
- implement DatabaseCommit for DatabaseComponents ([#1163](https://github.com/megaeth-labs/revm/pull/1163))
- add PrecompileError::Other ([#1165](https://github.com/megaeth-labs/revm/pull/1165))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/megaeth-labs/revm/pull/1157))
- Add boxed precompile trait ([#1131](https://github.com/megaeth-labs/revm/pull/1131))
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/megaeth-labs/revm/pull/1106))
- tweeks for v4.0 revm release ([#1048](https://github.com/megaeth-labs/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/megaeth-labs/revm/pull/1009))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/megaeth-labs/revm/pull/990))
- Convert optimism panic into graceful error ([#982](https://github.com/megaeth-labs/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/megaeth-labs/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/megaeth-labs/revm/pull/972))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/megaeth-labs/revm/pull/871))
- Loop call stack ([#851](https://github.com/megaeth-labs/revm/pull/851))
- transition account balance delta ([#843](https://github.com/megaeth-labs/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/megaeth-labs/revm/pull/834))
- add is_empty_code_hash fn ([#826](https://github.com/megaeth-labs/revm/pull/826))
- *(revme)* format kzg setup ([#818](https://github.com/megaeth-labs/revm/pull/818))
- add more `auto_impl`s to revm traits ([#799](https://github.com/megaeth-labs/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/megaeth-labs/revm/pull/794))
- add changed storage slots iter ([#801](https://github.com/megaeth-labs/revm/pull/801))
- derive more traits ([#745](https://github.com/megaeth-labs/revm/pull/745))
- Alloy primitives ([#724](https://github.com/megaeth-labs/revm/pull/724))
- Optimism execution changes ([#682](https://github.com/megaeth-labs/revm/pull/682))
- add "kzg" as a separate feature ([#746](https://github.com/megaeth-labs/revm/pull/746))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- derive PartialEq, Eq for Env ([#689](https://github.com/megaeth-labs/revm/pull/689))
- Optional coinbase tip ([#625](https://github.com/megaeth-labs/revm/pull/625))
- *(state)* Block hash cache and overrides ([#621](https://github.com/megaeth-labs/revm/pull/621))
- alloy migration ([#535](https://github.com/megaeth-labs/revm/pull/535))
- State with account status ([#499](https://github.com/megaeth-labs/revm/pull/499))
- introduce initcode size limit check taking config into account ([#587](https://github.com/megaeth-labs/revm/pull/587))
- EIP-1153 Transient storage opcodes ([#546](https://github.com/megaeth-labs/revm/pull/546))
- Implement `Error` for EVMError ([#559](https://github.com/megaeth-labs/revm/pull/559))
- *(cancun)* EIP-5656: MCOPY - Memory copying instruction ([#528](https://github.com/megaeth-labs/revm/pull/528))
- separate initial checks ([#486](https://github.com/megaeth-labs/revm/pull/486))
- Introduce account status as bitflag inside JournalState ([#477](https://github.com/megaeth-labs/revm/pull/477))
- add Output::into_data ([#420](https://github.com/megaeth-labs/revm/pull/420))
- Add all internals results to Halt ([#413](https://github.com/megaeth-labs/revm/pull/413))
- use singular bytes for the jumpmap ([#402](https://github.com/megaeth-labs/revm/pull/402))
- extend SuccessOrHalt ([#405](https://github.com/megaeth-labs/revm/pull/405))
- Add check for chainID ([#393](https://github.com/megaeth-labs/revm/pull/393))
- add b256<->u256 from impls ([#398](https://github.com/megaeth-labs/revm/pull/398))
- add API to retrieve unpadded bytecode ([#397](https://github.com/megaeth-labs/revm/pull/397))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- implement Debug for DatabaseComponentError if supported ([#363](https://github.com/megaeth-labs/revm/pull/363))
- Different OutOfGas Error types ([#354](https://github.com/megaeth-labs/revm/pull/354))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))

### Fixed

- *(journal)* emit EIP-7708 log for selfdestructed accounts with remaining balance ([#3394](https://github.com/megaeth-labs/revm/pull/3394))
- Align no_std OnceLock semantics with std ([#3026](https://github.com/megaeth-labs/revm/pull/3026))
- racecondition return on install_crypto fn ([#2997](https://github.com/megaeth-labs/revm/pull/2997))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- features and check in ci ([#2766](https://github.com/megaeth-labs/revm/pull/2766))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- correct propagate features ([#2177](https://github.com/megaeth-labs/revm/pull/2177))
- *(EOF)* Overflow on num_sections ([#1656](https://github.com/megaeth-labs/revm/pull/1656))
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/megaeth-labs/revm/pull/1605))
- *(EOF)* Use cfg code size limit for eofcreate ([#1606](https://github.com/megaeth-labs/revm/pull/1606))
- missing kzg_settings if kzg-rs feature enabled ([#1601](https://github.com/megaeth-labs/revm/pull/1601))
- *(eof)* fixture 2 tests ([#1550](https://github.com/megaeth-labs/revm/pull/1550))
- *(primitives)* specify the optimism cfg on spec_to_generic ([#1412](https://github.com/megaeth-labs/revm/pull/1412))
- fix eip3155 summary gas_used bug and add fork name ([#1216](https://github.com/megaeth-labs/revm/pull/1216))
- use std hashmap with "std" feature ([#1193](https://github.com/megaeth-labs/revm/pull/1193))
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/megaeth-labs/revm/pull/1098))
- optimism compilation ([#1091](https://github.com/megaeth-labs/revm/pull/1091))
- Set is_optimism to true if default config ([#1087](https://github.com/megaeth-labs/revm/pull/1087))
- use maximum possible data fee for 4844 balance checks ([#981](https://github.com/megaeth-labs/revm/pull/981))
- make revm-primitives no-std with c-kzg feature ([#933](https://github.com/megaeth-labs/revm/pull/933))
- *(ci)* Workflow Touchups ([#901](https://github.com/megaeth-labs/revm/pull/901))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/megaeth-labs/revm/pull/824))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/megaeth-labs/revm/pull/795))
- *(primitives)* Error Primitive Display + Error Implementations ([#770](https://github.com/megaeth-labs/revm/pull/770))
- use u128 for calc_blob_gasprice ([#764](https://github.com/megaeth-labs/revm/pull/764))
- use u128 for calc data fee result ([#757](https://github.com/megaeth-labs/revm/pull/757))
- balance check disabled ([#751](https://github.com/megaeth-labs/revm/pull/751))
- *(eip4844)* Pass eth tests, additional conditions added. ([#735](https://github.com/megaeth-labs/revm/pull/735))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))
- compile errors for features ([#467](https://github.com/megaeth-labs/revm/pull/467))
- *(interpreter)* halt on CreateInitcodeSizeLimit ([#412](https://github.com/megaeth-labs/revm/pull/412))

### Other

- release prep — bump all crates with unpublished changes ([#3721](https://github.com/megaeth-labs/revm/pull/3721))
- explain CPSB acronym ([#3716](https://github.com/megaeth-labs/revm/pull/3716))
- release ([#3679](https://github.com/megaeth-labs/revm/pull/3679))
- remove unused spec ids ([#3649](https://github.com/megaeth-labs/revm/pull/3649))
- audit #[allow] attributes ([#3611](https://github.com/megaeth-labs/revm/pull/3611))
- *(precompile)* use static OnceLock array for Precompiles init ([#3602](https://github.com/megaeth-labs/revm/pull/3602))
- *(primitives)* add SpecId::NEXT, drop num_enum dependency ([#3593](https://github.com/megaeth-labs/revm/pull/3593))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/megaeth-labs/revm/pull/3592))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/megaeth-labs/revm/pull/3568))
- release ([#3472](https://github.com/megaeth-labs/revm/pull/3472))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/megaeth-labs/revm/pull/3358))
- update default hardfork to Osaka (Ethereum) and Jovian (Optimism) ([#3326](https://github.com/megaeth-labs/revm/pull/3326))
- revm v34.0.0 ([#3313](https://github.com/megaeth-labs/revm/pull/3313))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/megaeth-labs/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/megaeth-labs/revm/pull/3272))
- *(database)* use fixed hashmaps in cache db ([#3231](https://github.com/megaeth-labs/revm/pull/3231))
- *(fmt)* merge all imports ([#3184](https://github.com/megaeth-labs/revm/pull/3184))
- merge v98 versions bumps ([#3155](https://github.com/megaeth-labs/revm/pull/3155))
- tag v96 revm v31.0.0 ([#3135](https://github.com/megaeth-labs/revm/pull/3135))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- resize short addresses bitvec instead of reallocating ([#3083](https://github.com/megaeth-labs/revm/pull/3083))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- add amsterdam in spec id ([#2934](https://github.com/megaeth-labs/revm/pull/2934))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/megaeth-labs/revm/pull/2925))
- cargo update ([#2930](https://github.com/megaeth-labs/revm/pull/2930))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- small performance and safety improvements ([#2868](https://github.com/megaeth-labs/revm/pull/2868))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- improve primitives crate documentation and consistency ([#2829](https://github.com/megaeth-labs/revm/pull/2829))
- reuse global crypto provide idea ([#2786](https://github.com/megaeth-labs/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- add OnceLock re-export with no_std support ([#2787](https://github.com/megaeth-labs/revm/pull/2787))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- support functions for eip7918 ([#2579](https://github.com/megaeth-labs/revm/pull/2579))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- nit, activation timestamp was 84, first 7702 bundle landed in 86 ([#2528](https://github.com/megaeth-labs/revm/pull/2528))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- add Prague activation timestamp ([#2514](https://github.com/megaeth-labs/revm/pull/2514))
- Storage Types Alias ([#2461](https://github.com/megaeth-labs/revm/pull/2461))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- replaced SpecId enumn with num_enum ([#2420](https://github.com/megaeth-labs/revm/pull/2420))
- bump stable tests, introduce lints ([#2403](https://github.com/megaeth-labs/revm/pull/2403))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/megaeth-labs/revm/pull/2345))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- Remove LATEST variant from SpecId enum ([#2299](https://github.com/megaeth-labs/revm/pull/2299))
- make number more readable ([#2300](https://github.com/megaeth-labs/revm/pull/2300))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- v65 revm: v20.0.0-alpha.7 ([#2280](https://github.com/megaeth-labs/revm/pull/2280))
- make str to SpecId conversion fallible ([#2236](https://github.com/megaeth-labs/revm/pull/2236))
- remove blockhash windows const ([#2228](https://github.com/megaeth-labs/revm/pull/2228))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- backport op l1 fetch perf ([#2076](https://github.com/megaeth-labs/revm/pull/2076))
- Bump licence year to 2025 ([#2058](https://github.com/megaeth-labs/revm/pull/2058))
- align crates versions ([#1983](https://github.com/megaeth-labs/revm/pull/1983))
- Update HISTORY_STORAGE_ADDRESS ([#1946](https://github.com/megaeth-labs/revm/pull/1946))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
- Update README.md examples section ([#1853](https://github.com/megaeth-labs/revm/pull/1853))
- *(primitives)* replace HashMap re-exports with alloy_primitives::map ([#1805](https://github.com/megaeth-labs/revm/pull/1805))
- *(deps)* bump alloy-primitives from 0.8.0 to 0.8.2 ([#1761](https://github.com/megaeth-labs/revm/pull/1761))
- release ([#1729](https://github.com/megaeth-labs/revm/pull/1729))
- Bump new logo ([#1735](https://github.com/megaeth-labs/revm/pull/1735))
- bump kzg-rs version ([#1734](https://github.com/megaeth-labs/revm/pull/1734))
- release ([#1722](https://github.com/megaeth-labs/revm/pull/1722))
- *(deps)* bump alloy and primitives ([#1725](https://github.com/megaeth-labs/revm/pull/1725))
- cast block number to u64 and not usize ([#1727](https://github.com/megaeth-labs/revm/pull/1727))
- bump `kzg-rs` version ([#1726](https://github.com/megaeth-labs/revm/pull/1726))
- release ([#1683](https://github.com/megaeth-labs/revm/pull/1683))
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/megaeth-labs/revm/pull/1685))
- Renamed some city name ([#1645](https://github.com/megaeth-labs/revm/pull/1645))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/megaeth-labs/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/megaeth-labs/revm/pull/1621))
- *(eof)* avoid some allocations ([#1632](https://github.com/megaeth-labs/revm/pull/1632))
- *(eof)* simplify magic checks ([#1633](https://github.com/megaeth-labs/revm/pull/1633))
- bump versions bcs of primitives ([#1631](https://github.com/megaeth-labs/revm/pull/1631))
- *(EOF)* Use Bytecode::new_legacy ([#1628](https://github.com/megaeth-labs/revm/pull/1628))
- release ([#1620](https://github.com/megaeth-labs/revm/pull/1620))
- bump alloy deps ([#1623](https://github.com/megaeth-labs/revm/pull/1623))
- *(deps)* bump alloy-primitives from 0.7.6 to 0.7.7 ([#1612](https://github.com/megaeth-labs/revm/pull/1612))
- group optimism invalid txn errors ([#1604](https://github.com/megaeth-labs/revm/pull/1604))
- *(deps)* bump bitflags from 2.5.0 to 2.6.0 ([#1583](https://github.com/megaeth-labs/revm/pull/1583))
- Rename gas_price to gas_limit for precompile args ([#1593](https://github.com/megaeth-labs/revm/pull/1593))
- release ([#1579](https://github.com/megaeth-labs/revm/pull/1579))
- *(README)* add rbuilder to used-by ([#1585](https://github.com/megaeth-labs/revm/pull/1585))
- add utility function AccountInfo::from_bytecode ([#1577](https://github.com/megaeth-labs/revm/pull/1577))
- replace AccessList with alloy version ([#1552](https://github.com/megaeth-labs/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/megaeth-labs/revm/pull/1505))
- release ([#1548](https://github.com/megaeth-labs/revm/pull/1548))
- replace TransactTo with TxKind ([#1542](https://github.com/megaeth-labs/revm/pull/1542))
- remove DatabaseWithDebugError ([#1545](https://github.com/megaeth-labs/revm/pull/1545))
- avoid cloning precompiles ([#1486](https://github.com/megaeth-labs/revm/pull/1486))
- added simular to used-by ([#1521](https://github.com/megaeth-labs/revm/pull/1521))
- derive PartialEq and Hash on EnvKzgSettings ([#1494](https://github.com/megaeth-labs/revm/pull/1494))
- remove old deprecated items ([#1489](https://github.com/megaeth-labs/revm/pull/1489))
- *(primitives)* rename State/Storage to EvmState/EvmStorage ([#1459](https://github.com/megaeth-labs/revm/pull/1459))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/megaeth-labs/revm/pull/1354))" ([#1424](https://github.com/megaeth-labs/revm/pull/1424))" ([#1426](https://github.com/megaeth-labs/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/megaeth-labs/revm/pull/1354))" ([#1424](https://github.com/megaeth-labs/revm/pull/1424))
- release ([#1261](https://github.com/megaeth-labs/revm/pull/1261))
- add Trin to used by list ([#1393](https://github.com/megaeth-labs/revm/pull/1393))
- bump c-kzg to 1.0.2 ([#1390](https://github.com/megaeth-labs/revm/pull/1390))
- refactor lints ([#1386](https://github.com/megaeth-labs/revm/pull/1386))
- add blob_count and max_blobs to `TooManyBlobs` err enum ([#1375](https://github.com/megaeth-labs/revm/pull/1375))
- bump alloy & specify dep rev ([#1380](https://github.com/megaeth-labs/revm/pull/1380))
- don't clone bytes in `Bytecode::bytes` ([#1344](https://github.com/megaeth-labs/revm/pull/1344))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/megaeth-labs/revm/pull/1307))
- Implement `with_chain_id` for `CfgEnv` ([#1327](https://github.com/megaeth-labs/revm/pull/1327))
- *(interpreter)* remove EOF branch in CODE{SIZE,COPY} ([#1308](https://github.com/megaeth-labs/revm/pull/1308))
- Update documentation ([#1275](https://github.com/megaeth-labs/revm/pull/1275))
- release ([#1231](https://github.com/megaeth-labs/revm/pull/1231))
- use uint macro & fix various small things ([#1253](https://github.com/megaeth-labs/revm/pull/1253))
- *(deps)* bump alloy 0.7.0 ([#1250](https://github.com/megaeth-labs/revm/pull/1250))
- Revert "feat: optional nonce check ([#1195](https://github.com/megaeth-labs/revm/pull/1195))" ([#1212](https://github.com/megaeth-labs/revm/pull/1212))
- release ([#1175](https://github.com/megaeth-labs/revm/pull/1175))
- *(deps)* bump bitflags from 2.4.2 to 2.5.0 ([#1210](https://github.com/megaeth-labs/revm/pull/1210))
- clarify None nonce ([#1204](https://github.com/megaeth-labs/revm/pull/1204))
- c-kzg v1.0.0 ([#1190](https://github.com/megaeth-labs/revm/pull/1190))
- fix some typos ([#1189](https://github.com/megaeth-labs/revm/pull/1189))
- Fix typo in readme ([#1185](https://github.com/megaeth-labs/revm/pull/1185))
- improve error implementations ([#1183](https://github.com/megaeth-labs/revm/pull/1183))
- release ([#1125](https://github.com/megaeth-labs/revm/pull/1125))
- Add Hardhat to the "Used by" list ([#1164](https://github.com/megaeth-labs/revm/pull/1164))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/megaeth-labs/revm/pull/1140))
- Add VERBS to used by list ([#1141](https://github.com/megaeth-labs/revm/pull/1141))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/megaeth-labs/revm/pull/1132))
- add helper function for spec id ([#1130](https://github.com/megaeth-labs/revm/pull/1130))
- release tag v30 revm v6.1.0 ([#1100](https://github.com/megaeth-labs/revm/pull/1100))
- clippy cleanup ([#1112](https://github.com/megaeth-labs/revm/pull/1112))
- release ([#1082](https://github.com/megaeth-labs/revm/pull/1082))
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/megaeth-labs/revm/pull/1085))
- license date and revm docs ([#1080](https://github.com/megaeth-labs/revm/pull/1080))
- tag v27, revm v4.0.0 release ([#1061](https://github.com/megaeth-labs/revm/pull/1061))
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/megaeth-labs/revm/pull/1059))
- helper functions around Env ([#1057](https://github.com/megaeth-labs/revm/pull/1057))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/megaeth-labs/revm/pull/1041))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/megaeth-labs/revm/pull/1026))
- add some docs to StorageSlot ([#1019](https://github.com/megaeth-labs/revm/pull/1019))
- fix serde std flags for no-std build ([#987](https://github.com/megaeth-labs/revm/pull/987))
- *(deps)* bump bitflags from 2.4.1 to 2.4.2 ([#983](https://github.com/megaeth-labs/revm/pull/983))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/megaeth-labs/revm/pull/975))
- enhance readability ([#968](https://github.com/megaeth-labs/revm/pull/968))
- *(interpreter)* improve enum naming ([#962](https://github.com/megaeth-labs/revm/pull/962))
- *(deps)* alloy 0.6 ([#963](https://github.com/megaeth-labs/revm/pull/963))
- *(primitives)* improve readability of Env.validate_tx() ([#924](https://github.com/megaeth-labs/revm/pull/924))
- *(primitives)* optimize AccountInfo.is_empty() ([#922](https://github.com/megaeth-labs/revm/pull/922))
- *(deps)* bump once_cell from 1.18.0 to 1.19.0 ([#908](https://github.com/megaeth-labs/revm/pull/908))
- *(docs)* Update the benchmark docs to point to revm package ([#906](https://github.com/megaeth-labs/revm/pull/906))
- *(docs)* Update top-level benchmark docs ([#894](https://github.com/megaeth-labs/revm/pull/894))
- fix case for CreateInitCodeSizeLimit error ([#896](https://github.com/megaeth-labs/revm/pull/896))
- Add docs to `optimism` `InvalidTransaction` errors. ([#884](https://github.com/megaeth-labs/revm/pull/884))
- *(deps)* run `cargo update`, bump alloy-primitives ([#880](https://github.com/megaeth-labs/revm/pull/880))
- *(primitives)* don't duplicate the SpecId::enabled() logic ([#869](https://github.com/megaeth-labs/revm/pull/869))
- simplify use statements ([#864](https://github.com/megaeth-labs/revm/pull/864))
- cargo fmt and removed extra newlines ([#860](https://github.com/megaeth-labs/revm/pull/860))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/megaeth-labs/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/megaeth-labs/revm/pull/859))
- bump c-kzg to v0.4.0 ([#849](https://github.com/megaeth-labs/revm/pull/849))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/megaeth-labs/revm/pull/844))
- decode KZG points directly into the buffers ([#840](https://github.com/megaeth-labs/revm/pull/840))
- *(deps)* make derive-more optional ([#827](https://github.com/megaeth-labs/revm/pull/827))
- *(eip4844)* update kzg trusted setup ([#822](https://github.com/megaeth-labs/revm/pull/822))
- *(deps)* bump bitflags from 2.4.0 to 2.4.1 ([#803](https://github.com/megaeth-labs/revm/pull/803))
- include alloy-primitives std feature ([#788](https://github.com/megaeth-labs/revm/pull/788))
- clang requirement ([#784](https://github.com/megaeth-labs/revm/pull/784))
- *(primitives)* OptimismFields Docs Cleanup ([#778](https://github.com/megaeth-labs/revm/pull/778))
- Remove dead state trait function ([#779](https://github.com/megaeth-labs/revm/pull/779))
- Verify account is empty checks ([#780](https://github.com/megaeth-labs/revm/pull/780))
- *(primitives)* Clean up the account typs with docs ([#776](https://github.com/megaeth-labs/revm/pull/776))
- Use upstream create and create2 implementations ([#775](https://github.com/megaeth-labs/revm/pull/775))
- Implement display and error for precompile error ([#777](https://github.com/megaeth-labs/revm/pull/777))
- reorder env members ([#771](https://github.com/megaeth-labs/revm/pull/771))
- document everything, dedup existing docs ([#741](https://github.com/megaeth-labs/revm/pull/741))
- bump v26 revm v3.5.0 ([#765](https://github.com/megaeth-labs/revm/pull/765))
- *(deps)* bump proptest from 1.2.0 to 1.3.1 ([#763](https://github.com/megaeth-labs/revm/pull/763))
- tag v25, revm v3.4.0 ([#755](https://github.com/megaeth-labs/revm/pull/755))
- bump c-kzg to create lib ([#758](https://github.com/megaeth-labs/revm/pull/758))
- Readme Updates ([#756](https://github.com/megaeth-labs/revm/pull/756))
- Improve wording and fix typos ([#749](https://github.com/megaeth-labs/revm/pull/749))
- Logo ([#743](https://github.com/megaeth-labs/revm/pull/743))
- BLOBBASEFEE opcode ([#721](https://github.com/megaeth-labs/revm/pull/721))
- error type for block header ([#731](https://github.com/megaeth-labs/revm/pull/731))
- document when InvalidTransaction errors are thrown ([#722](https://github.com/megaeth-labs/revm/pull/722))
- rm nonexhaustive for error ([#726](https://github.com/megaeth-labs/revm/pull/726))
- clippy incorrect_clone_impl_on_copy_type ([#720](https://github.com/megaeth-labs/revm/pull/720))
- *(deps)* bump bytes from 1.4.0 to 1.5.0 ([#707](https://github.com/megaeth-labs/revm/pull/707))
- implement `Default` for other databases ([#691](https://github.com/megaeth-labs/revm/pull/691))
- *(cfg)* convert chain_id from u256 to u64 ([#693](https://github.com/megaeth-labs/revm/pull/693))
- Small doc comment fix ([#698](https://github.com/megaeth-labs/revm/pull/698))
- deprecate `RefDBWrapper` ([#696](https://github.com/megaeth-labs/revm/pull/696))
- remove unused new_raw_with_hash ([#676](https://github.com/megaeth-labs/revm/pull/676))
- Removed the last dependencies breaking no-std build. ([#669](https://github.com/megaeth-labs/revm/pull/669))
- *(deps)* bump proptest-derive from 0.3.0 to 0.4.0 ([#652](https://github.com/megaeth-labs/revm/pull/652))
- misc improvements ([#633](https://github.com/megaeth-labs/revm/pull/633))
- bundle state ([#637](https://github.com/megaeth-labs/revm/pull/637))
- book workflow ([#537](https://github.com/megaeth-labs/revm/pull/537))
- mark CfgEnv as non_exhaustive ([#623](https://github.com/megaeth-labs/revm/pull/623))
- Revert "feat: alloy migration ([#535](https://github.com/megaeth-labs/revm/pull/535))" ([#616](https://github.com/megaeth-labs/revm/pull/616))
- spell check ([#615](https://github.com/megaeth-labs/revm/pull/615))
- export some `unreachable_pub` items ([#598](https://github.com/megaeth-labs/revm/pull/598))
- Revert test, not change storage check , renaming of original slot value ([#601](https://github.com/megaeth-labs/revm/pull/601))
- avoid unnecessary allocations ([#581](https://github.com/megaeth-labs/revm/pull/581))
- *(deps)* bump bitflags from 2.3.3 to 2.4.0 ([#596](https://github.com/megaeth-labs/revm/pull/596))
- clippy and fmt ([#568](https://github.com/megaeth-labs/revm/pull/568))
- *(deps)* bump hashbrown from 0.13.2 to 0.14.0 ([#519](https://github.com/megaeth-labs/revm/pull/519))
- *(deps)* bump auto_impl from 1.0.1 to 1.1.0 ([#478](https://github.com/megaeth-labs/revm/pull/478))
- add util functions for getting output data ([#509](https://github.com/megaeth-labs/revm/pull/509))
- fix typo ([#488](https://github.com/megaeth-labs/revm/pull/488))
- Bundle inspector crate/call calls ([#480](https://github.com/megaeth-labs/revm/pull/480))
- Bump v24, revm v3.3.0 ([#476](https://github.com/megaeth-labs/revm/pull/476))
- *(deps)* bump ruint from 1.7.0 to 1.8.0 ([#465](https://github.com/megaeth-labs/revm/pull/465))
- Fix typo in primitives/src/state.rs ([#474](https://github.com/megaeth-labs/revm/pull/474))
- add example to revm crate ([#468](https://github.com/megaeth-labs/revm/pull/468))
- Release v22, revm v3.1.1 ([#460](https://github.com/megaeth-labs/revm/pull/460))
- add into_logs ([#453](https://github.com/megaeth-labs/revm/pull/453))
- v21, revm v3.1.0 ([#444](https://github.com/megaeth-labs/revm/pull/444))
- bump all
- add feature for ignoring base fee check ([#436](https://github.com/megaeth-labs/revm/pull/436))
- Comment Fix ([#430](https://github.com/megaeth-labs/revm/pull/430))
- *(deps)* bump arbitrary from 1.2.3 to 1.3.0 ([#428](https://github.com/megaeth-labs/revm/pull/428))
- Update README.md ([#424](https://github.com/megaeth-labs/revm/pull/424))
- enabled primtive default feature in precompile ([#409](https://github.com/megaeth-labs/revm/pull/409))
- Update readmdoc of `perf_analyse_created_bytecodes` ([#404](https://github.com/megaeth-labs/revm/pull/404))
- remove gas blocks ([#391](https://github.com/megaeth-labs/revm/pull/391))
- impl NonceTooHigh/ NonceTooLow checks ([#383](https://github.com/megaeth-labs/revm/pull/383))
- derive Debug for DatabaseComponentError ([#377](https://github.com/megaeth-labs/revm/pull/377))
- Add B160/B256 From primitive_types traits ([#380](https://github.com/megaeth-labs/revm/pull/380))
- add no_std to primitives ([#366](https://github.com/megaeth-labs/revm/pull/366))
- *(deps)* bump bytes from 1.3.0 to 1.4.0 ([#355](https://github.com/megaeth-labs/revm/pull/355))
- rename Then to Than ([#368](https://github.com/megaeth-labs/revm/pull/368))
- use alloc & core for Arc impl ([#367](https://github.com/megaeth-labs/revm/pull/367))
- add error details to InvalidTransaction::LackOfFundForGasLimit ([#364](https://github.com/megaeth-labs/revm/pull/364))
- implement BlockHash for Arc<BlockHashRef> ([#361](https://github.com/megaeth-labs/revm/pull/361))
- implement State for Arc<StateRef> ([#360](https://github.com/megaeth-labs/revm/pull/360))
- *(deps)* bump proptest from 1.0.0 to 1.1.0 ([#358](https://github.com/megaeth-labs/revm/pull/358))
- revm-precompiles to revm-precompile
- nits and num of cargo keyword to 5 ([#351](https://github.com/megaeth-labs/revm/pull/351))
- Bump v20, changelog ([#350](https://github.com/megaeth-labs/revm/pull/350))
- Cleanup imports ([#348](https://github.com/megaeth-labs/revm/pull/348))
- add logs & return value to revert ([#343](https://github.com/megaeth-labs/revm/pull/343))
- expose hashbrown::HashMap in primitives ([#345](https://github.com/megaeth-labs/revm/pull/345))
- simplify DatabaseComponentError ([#339](https://github.com/megaeth-labs/revm/pull/339))
- includes to libs ([#338](https://github.com/megaeth-labs/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/megaeth-labs/revm/pull/334))
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

## [24.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v24.0.0...revm-primitives-v24.0.1) - 2026-05-26

### Other

- explain CPSB acronym ([#3716](https://github.com/bluealloy/revm/pull/3716))

## [24.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v23.0.0...revm-primitives-v24.0.0) - 2026-05-19

### Added

- *(eip8037)* Amsterdam bal-devnet-7 ([#3667](https://github.com/bluealloy/revm/pull/3667))

### Other

- remove unused spec ids ([#3649](https://github.com/bluealloy/revm/pull/3649))
- audit #[allow] attributes ([#3611](https://github.com/bluealloy/revm/pull/3611))
- *(precompile)* use static OnceLock array for Precompiles init ([#3602](https://github.com/bluealloy/revm/pull/3602))
- *(primitives)* add SpecId::NEXT, drop num_enum dependency ([#3593](https://github.com/bluealloy/revm/pull/3593))
- enable and fix clippy::missing_const_for_fn ([#3592](https://github.com/bluealloy/revm/pull/3592))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [23.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v22.1.0...revm-primitives-v23.0.0) - 2026-04-10

### Added

- Part of amsterdam devnet3 EIP updates ([#3438](https://github.com/bluealloy/revm/pull/3438))

## [22.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v22.0.0...revm-primitives-v22.1.0) - 2026-03-02

### Added

- Implement EIP-7708 ETH transfers emit a log ([#3334](https://github.com/bluealloy/revm/pull/3334))

### Fixed

- *(journal)* emit EIP-7708 log for selfdestructed accounts with remaining balance ([#3394](https://github.com/bluealloy/revm/pull/3394))

### Other

- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))
- use fixed bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))
- update default hardfork to Osaka (Ethereum) and Jovian (Optimism) ([#3326](https://github.com/bluealloy/revm/pull/3326))

## [21.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v21.0.2...revm-primitives-v21.1.0) - 2026-01-15

### Added

- BAL EIP-7928 ([#3070](https://github.com/bluealloy/revm/pull/3070))
- JournaledAccount sload/sstore ([#3201](https://github.com/bluealloy/revm/pull/3201))

### Other

- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- *(database)* use fixed hashmaps in cache db ([#3231](https://github.com/bluealloy/revm/pull/3231))
- *(fmt)* merge all imports ([#3184](https://github.com/bluealloy/revm/pull/3184))

## [21.0.2](https://github.com/bluealloy/revm/compare/revm-primitives-v21.0.1...revm-primitives-v21.0.2) - 2025-11-07

### Other

- tag v96 revm v31.0.0 ([#3135](https://github.com/bluealloy/revm/pull/3135))

## [21.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v21.0.0...revm-primitives-v21.0.1) - 2025-10-15

### Other

- resize short addresses bitvec instead of reallocating ([#3083](https://github.com/bluealloy/revm/pull/3083))

## [21.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v20.2.1...revm-primitives-v21.0.0) - 2025-10-07

### Added

- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- Align no_std OnceLock semantics with std ([#3026](https://github.com/bluealloy/revm/pull/3026))
- racecondition return on install_crypto fn ([#2997](https://github.com/bluealloy/revm/pull/2997))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- add amsterdam in spec id ([#2934](https://github.com/bluealloy/revm/pull/2934))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/bluealloy/revm/pull/2925))
- cargo update ([#2930](https://github.com/bluealloy/revm/pull/2930))

## [20.2.1](https://github.com/bluealloy/revm/compare/revm-primitives-v20.2.0...revm-primitives-v20.2.1) - 2025-08-12

### Other

- small performance and safety improvements ([#2868](https://github.com/bluealloy/revm/pull/2868))

## [20.2.0](https://github.com/bluealloy/revm/compare/revm-primitives-v20.1.0...revm-primitives-v20.2.0) - 2025-08-06

### Added

- short address for journal cold/warm check ([#2849](https://github.com/bluealloy/revm/pull/2849))

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- improve primitives crate documentation and consistency ([#2829](https://github.com/bluealloy/revm/pull/2829))
- reuse global crypto provide idea ([#2786](https://github.com/bluealloy/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
- add OnceLock re-export with no_std support ([#2787](https://github.com/bluealloy/revm/pull/2787))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [20.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v20.0.0...revm-primitives-v20.1.0) - 2025-07-23

### Added

- *(osaka)* update EIP-7825 constant ([#2753](https://github.com/bluealloy/revm/pull/2753))
- expose sha3-keccak in revm and revm-primitives ([#2713](https://github.com/bluealloy/revm/pull/2713))

### Fixed

- features and check in ci ([#2766](https://github.com/bluealloy/revm/pull/2766))

## [20.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v19.2.0...revm-primitives-v20.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/bluealloy/revm/pull/2611)) ([#2642](https://github.com/bluealloy/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))

## [19.2.0](https://github.com/bluealloy/revm/compare/revm-primitives-v19.1.0...revm-primitives-v19.2.0) - 2025-06-06

### Added

- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/bluealloy/revm/pull/2575))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- support functions for eip7918 ([#2579](https://github.com/bluealloy/revm/pull/2579))

## [19.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v19.0.0...revm-primitives-v19.1.0) - 2025-05-22

### Added

- *(Osaka)* modexp input limit and gas change, EIP-7823 and EIP-7883 ([#2531](https://github.com/bluealloy/revm/pull/2531))

### Other

- nit, activation timestamp was 84, first 7702 bundle landed in 86 ([#2528](https://github.com/bluealloy/revm/pull/2528))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- add Prague activation timestamp ([#2514](https://github.com/bluealloy/revm/pull/2514))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [19.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v18.0.0...revm-primitives-v19.0.0) - 2025-05-07

### Added

- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- replaced SpecId enumn with num_enum ([#2420](https://github.com/bluealloy/revm/pull/2420))
- bump stable tests, introduce lints ([#2403](https://github.com/bluealloy/revm/pull/2403))

## [18.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v17.0.0...revm-primitives-v18.0.0) - 2025-04-09

### Other

- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0...revm-primitives-v17.0.0) - 2025-03-28

### Other

- Remove LATEST variant from SpecId enum ([#2299](https://github.com/bluealloy/revm/pull/2299))
- make number more readable ([#2300](https://github.com/bluealloy/revm/pull/2300))

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0-alpha.5...revm-primitives-v16.0.0) - 2025-03-24

Stable version

## [16.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0-alpha.4...revm-primitives-v16.0.0-alpha.5) - 2025-03-21

### Other

- make str to SpecId conversion fallible ([#2236](https://github.com/bluealloy/revm/pull/2236))
- remove blockhash windows const ([#2228](https://github.com/bluealloy/revm/pull/2228))

## [16.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0-alpha.3...revm-primitives-v16.0.0-alpha.4) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [16.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0-alpha.2...revm-primitives-v16.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [16.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-primitives-v16.0.0-alpha.1...revm-primitives-v16.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [16.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revm-primitives-v15.1.0...revm-primitives-v16.0.0-alpha.1) - 2025-02-16

### Added

- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- simplify Transaction trait (#1959)
- restructuring Part6 transaction crate (#1814)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- to_plain_state (#1778)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Other

- backport op l1 fetch perf (#2076)
- Bump licence year to 2025 (#2058)
- align crates versions (#1983)
- Update HISTORY_STORAGE_ADDRESS (#1946)
- fix comments and docs into more sensible (#1920)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)

## [15.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v14.0.0...revm-primitives-v15.1.0) - 2024-12-26

### Added

- blst reprice, remove g1/g2 mul, eest test bump ([#1951](https://github.com/bluealloy/revm/pull/1951))
- add Isthmus spec ([#1948](https://github.com/bluealloy/revm/pull/1948))
- eip7691 fraction update ([#1900](https://github.com/bluealloy/revm/pull/1900))
- apply latest EIP-7702 changes ([#1850](https://github.com/bluealloy/revm/pull/1850))
- *(Prague)* EIP-7623 Increase Calldata Cost ([#1744](https://github.com/bluealloy/revm/pull/1744))

### Other

- eip7702 chainid u256 change ([#1950](https://github.com/bluealloy/revm/pull/1950))
- Uncouple blob count between CL and EL ([#1899](https://github.com/bluealloy/revm/pull/1899))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v13.0.0...revm-primitives-v14.0.0) - 2024-11-06

### Added

- to_plain_state ([#1778](https://github.com/bluealloy/revm/pull/1778)) ([#1841](https://github.com/bluealloy/revm/pull/1841))

### Other

- bump alloy-eip7702 and remove `Parity` re-export ([#1842](https://github.com/bluealloy/revm/pull/1842))

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v12.0.0...revm-primitives-v13.0.0) - 2024-10-23

### Other

- bump alloy-eip7702 ([#1829](https://github.com/bluealloy/revm/pull/1829))

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v11.0.0...revm-primitives-v12.0.0) - 2024-10-17

### Other

- update Cargo.toml dependencies

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v10.0.0...revm-primitives-v11.0.0) - 2024-10-17

### Added

- EIP-7702 Add u8 validity ([#1824](https://github.com/bluealloy/revm/pull/1824))
- Rename PRAGUE_EOF to OSAKA ([#1822](https://github.com/bluealloy/revm/pull/1822))
- *(EIP-7702)* devnet-4 changes ([#1821](https://github.com/bluealloy/revm/pull/1821))

### Other

- bump newest primitives ([#1823](https://github.com/bluealloy/revm/pull/1823))

## [10.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v10.0.0...revm-primitives-v10.0.1) - 2024-09-26

### Other

- update Cargo.toml dependencies

## [9.0.2](https://github.com/bluealloy/revm/compare/revm-primitives-v9.0.1...revm-primitives-v9.0.2) - 2024-09-18

### Other

- *(deps)* bump alloy-primitives from 0.8.0 to 0.8.2 ([#1761](https://github.com/bluealloy/revm/pull/1761))

## [9.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v9.0.0...revm-primitives-v9.0.1) - 2024-08-30

### Other
- Bump new logo ([#1735](https://github.com/bluealloy/revm/pull/1735))
- bump kzg-rs version ([#1734](https://github.com/bluealloy/revm/pull/1734))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v8.0.0...revm-primitives-v9.0.0) - 2024-08-29

### Added
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/bluealloy/revm/pull/1695))
- c-kzg bump, cleanup on kzgsetting ([#1719](https://github.com/bluealloy/revm/pull/1719))

### Other
- *(deps)* bump alloy and primitives ([#1725](https://github.com/bluealloy/revm/pull/1725))
- cast block number to u64 and not usize ([#1727](https://github.com/bluealloy/revm/pull/1727))
- bump `kzg-rs` version ([#1726](https://github.com/bluealloy/revm/pull/1726))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v7.1.0...revm-primitives-v8.0.0) - 2024-08-08

### Added
- *(EOF)* add evmone test suite ([#1689](https://github.com/bluealloy/revm/pull/1689))
- check for typos in CI ([#1686](https://github.com/bluealloy/revm/pull/1686))
- *(EOF)* Add non-returning CALLF/JUMPF checks ([#1663](https://github.com/bluealloy/revm/pull/1663))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/bluealloy/revm/pull/1648))

### Fixed
- *(EOF)* Overflow on num_sections ([#1656](https://github.com/bluealloy/revm/pull/1656))

### Other
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/bluealloy/revm/pull/1685))
- Renamed some city name ([#1645](https://github.com/bluealloy/revm/pull/1645))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/bluealloy/revm/pull/1621))
- *(eof)* avoid some allocations ([#1632](https://github.com/bluealloy/revm/pull/1632))
- *(eof)* simplify magic checks ([#1633](https://github.com/bluealloy/revm/pull/1633))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v6.0.0...revm-primitives-v7.0.0) - 2024-07-16

### Added
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bluealloy/revm/pull/1607))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/bluealloy/revm/pull/1558))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))
- *(EOF)* Use cfg code size limit for eofcreate ([#1606](https://github.com/bluealloy/revm/pull/1606))
- missing kzg_settings if kzg-rs feature enabled ([#1601](https://github.com/bluealloy/revm/pull/1601))

### Other
- bump alloy deps ([#1623](https://github.com/bluealloy/revm/pull/1623))
- *(deps)* bump alloy-primitives from 0.7.6 to 0.7.7 ([#1612](https://github.com/bluealloy/revm/pull/1612))
- group optimism invalid txn errors ([#1604](https://github.com/bluealloy/revm/pull/1604))
- *(deps)* bump bitflags from 2.5.0 to 2.6.0 ([#1583](https://github.com/bluealloy/revm/pull/1583))
- Rename gas_price to gas_limit for precompile args ([#1593](https://github.com/bluealloy/revm/pull/1593))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v5.0.0...revm-primitives-v6.0.0) - 2024-07-08

### Added
- *(Precompiles)* Throw fatal error if c-kzg is disabled ([#1589](https://github.com/bluealloy/revm/pull/1589))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/bluealloy/revm/pull/1565))
- add helper function to mape EVMError's Database error variant ([#1567](https://github.com/bluealloy/revm/pull/1567))

### Other
- *(README)* add rbuilder to used-by ([#1585](https://github.com/bluealloy/revm/pull/1585))
- add utility function AccountInfo::from_bytecode ([#1577](https://github.com/bluealloy/revm/pull/1577))
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/bluealloy/revm/pull/1505))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v4.0.0...revm-primitives-v5.0.0) - 2024-06-20

### Added
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/bluealloy/revm/pull/1504))
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/bluealloy/revm/pull/1499))
- Persist reverted account and storage slot lookups in `JournaledState` ([#1437](https://github.com/bluealloy/revm/pull/1437))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/bluealloy/revm/pull/1467))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/bluealloy/revm/pull/1436))
- *(EOF)* Add CALLF/JUMPF stack checks ([#1417](https://github.com/bluealloy/revm/pull/1417))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/bluealloy/revm/pull/1415))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- *(primitives)* specify the optimism cfg on spec_to_generic ([#1412](https://github.com/bluealloy/revm/pull/1412))

### Other
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- remove DatabaseWithDebugError ([#1545](https://github.com/bluealloy/revm/pull/1545))
- avoid cloning precompiles ([#1486](https://github.com/bluealloy/revm/pull/1486))
- added simular to used-by ([#1521](https://github.com/bluealloy/revm/pull/1521))
- derive PartialEq and Hash on EnvKzgSettings ([#1494](https://github.com/bluealloy/revm/pull/1494))
- remove old deprecated items ([#1489](https://github.com/bluealloy/revm/pull/1489))
- *(primitives)* rename State/Storage to EvmState/EvmStorage ([#1459](https://github.com/bluealloy/revm/pull/1459))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))" ([#1426](https://github.com/bluealloy/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v3.1.1...revm-primitives-v4.0.0) - 2024-05-12

### Added
- Add bytes to Bytecode ([#1396](https://github.com/bluealloy/revm/pull/1396))
- implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))
- add `Bytecode::original_bytecode_slice` to match `BytecodeLocked` ([#1286](https://github.com/bluealloy/revm/pull/1286))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- pass rand feature to alloy_primitives ([#1276](https://github.com/bluealloy/revm/pull/1276))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/bluealloy/revm/pull/1243))

### Other
- add Trin to used by list ([#1393](https://github.com/bluealloy/revm/pull/1393))
- bump c-kzg to 1.0.2 ([#1390](https://github.com/bluealloy/revm/pull/1390))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- add blob_count and max_blobs to `TooManyBlobs` err enum ([#1375](https://github.com/bluealloy/revm/pull/1375))
- bump alloy & specify dep rev ([#1380](https://github.com/bluealloy/revm/pull/1380))
- don't clone bytes in `Bytecode::bytes` ([#1344](https://github.com/bluealloy/revm/pull/1344))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/bluealloy/revm/pull/1307))
- Implement `with_chain_id` for `CfgEnv` ([#1327](https://github.com/bluealloy/revm/pull/1327))
- *(interpreter)* remove EOF branch in CODE{SIZE,COPY} ([#1308](https://github.com/bluealloy/revm/pull/1308))
- Update documentation ([#1275](https://github.com/bluealloy/revm/pull/1275))

## [3.1.1](https://github.com/bluealloy/revm/compare/revm-primitives-v3.1.0...revm-primitives-v3.1.1) - 2024-04-02

### Fixed
- fix eip3155 summary gas_used bug and add fork name ([#1216](https://github.com/bluealloy/revm/pull/1216))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))
- *(deps)* bump alloy 0.7.0 ([#1250](https://github.com/bluealloy/revm/pull/1250))

## [3.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v3.0.0...revm-primitives-v3.1.0) - 2024-03-19

### Added
- implement Default for EnvWithHandlerCfg ([#1206](https://github.com/bluealloy/revm/pull/1206))
- optional nonce check ([#1195](https://github.com/bluealloy/revm/pull/1195))

### Fixed
- use std hashmap with "std" feature ([#1193](https://github.com/bluealloy/revm/pull/1193))

### Other
- *(deps)* bump bitflags from 2.4.2 to 2.5.0 ([#1210](https://github.com/bluealloy/revm/pull/1210))
- clarify None nonce ([#1204](https://github.com/bluealloy/revm/pull/1204))
- c-kzg v1.0.0 ([#1190](https://github.com/bluealloy/revm/pull/1190))
- fix some typos ([#1189](https://github.com/bluealloy/revm/pull/1189))
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))
- improve error implementations ([#1183](https://github.com/bluealloy/revm/pull/1183))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v2.1.0...revm-primitives-v3.0.0) - 2024-03-08

### Added
- *(primitives)* derive Default for SpecId ([#1171](https://github.com/bluealloy/revm/pull/1171))
- implement DatabaseCommit for DatabaseComponents ([#1163](https://github.com/bluealloy/revm/pull/1163))
- add PrecompileError::Other ([#1165](https://github.com/bluealloy/revm/pull/1165))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- Add boxed precompile trait ([#1131](https://github.com/bluealloy/revm/pull/1131))

### Other
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/bluealloy/revm/pull/1132))
- add helper function for spec id ([#1130](https://github.com/bluealloy/revm/pull/1130))

## [2.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v2.0.1...revm-primitives-v2.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))

### Other
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))

## [2.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v2.0.0...revm-primitives-v2.0.1) - 2024-02-17

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))
- optimism compilation ([#1091](https://github.com/bluealloy/revm/pull/1091))
- Set is_optimism to true if default config ([#1087](https://github.com/bluealloy/revm/pull/1087))

### Other
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v1.3.0...revm-primitives-v2.0.0) - 2024-02-07

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/bluealloy/revm/pull/990))
- Convert optimism panic into graceful error ([#982](https://github.com/bluealloy/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- transition account balance delta ([#843](https://github.com/bluealloy/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- add is_empty_code_hash fn ([#826](https://github.com/bluealloy/revm/pull/826))
- *(revme)* format kzg setup ([#818](https://github.com/bluealloy/revm/pull/818))
- add more `auto_impl`s to revm traits ([#799](https://github.com/bluealloy/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- add changed storage slots iter ([#801](https://github.com/bluealloy/revm/pull/801))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))

### Fixed
- use maximum possible data fee for 4844 balance checks ([#981](https://github.com/bluealloy/revm/pull/981))
- make revm-primitives no-std with c-kzg feature ([#933](https://github.com/bluealloy/revm/pull/933))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/bluealloy/revm/pull/824))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/bluealloy/revm/pull/795))
- *(primitives)* Error Primitive Display + Error Implementations ([#770](https://github.com/bluealloy/revm/pull/770))

### Other
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/bluealloy/revm/pull/1059))
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/bluealloy/revm/pull/1041))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/bluealloy/revm/pull/1026))
- add some docs to StorageSlot ([#1019](https://github.com/bluealloy/revm/pull/1019))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(deps)* bump bitflags from 2.4.1 to 2.4.2 ([#983](https://github.com/bluealloy/revm/pull/983))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(deps)* alloy 0.6 ([#963](https://github.com/bluealloy/revm/pull/963))
- *(primitives)* improve readability of Env.validate_tx() ([#924](https://github.com/bluealloy/revm/pull/924))
- *(primitives)* optimize AccountInfo.is_empty() ([#922](https://github.com/bluealloy/revm/pull/922))
- *(deps)* bump once_cell from 1.18.0 to 1.19.0 ([#908](https://github.com/bluealloy/revm/pull/908))
- fix case for CreateInitCodeSizeLimit error ([#896](https://github.com/bluealloy/revm/pull/896))
- Add docs to `optimism` `InvalidTransaction` errors. ([#884](https://github.com/bluealloy/revm/pull/884))
- *(deps)* run `cargo update`, bump alloy-primitives ([#880](https://github.com/bluealloy/revm/pull/880))
- *(primitives)* don't duplicate the SpecId::enabled() logic ([#869](https://github.com/bluealloy/revm/pull/869))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- cargo fmt and removed extra newlines ([#860](https://github.com/bluealloy/revm/pull/860))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/bluealloy/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/bluealloy/revm/pull/859))
- bump c-kzg to v0.4.0 ([#849](https://github.com/bluealloy/revm/pull/849))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/bluealloy/revm/pull/844))
- decode KZG points directly into the buffers ([#840](https://github.com/bluealloy/revm/pull/840))
- *(deps)* make derive-more optional ([#827](https://github.com/bluealloy/revm/pull/827))
- *(eip4844)* update kzg trusted setup ([#822](https://github.com/bluealloy/revm/pull/822))
- *(deps)* bump bitflags from 2.4.0 to 2.4.1 ([#803](https://github.com/bluealloy/revm/pull/803))
- include alloy-primitives std feature ([#788](https://github.com/bluealloy/revm/pull/788))
- *(primitives)* OptimismFields Docs Cleanup ([#778](https://github.com/bluealloy/revm/pull/778))
- Remove dead state trait function ([#779](https://github.com/bluealloy/revm/pull/779))
- Verify account is empty checks ([#780](https://github.com/bluealloy/revm/pull/780))
- *(primitives)* Clean up the account typs with docs ([#776](https://github.com/bluealloy/revm/pull/776))
- Use upstream create and create2 implementations ([#775](https://github.com/bluealloy/revm/pull/775))
- Implement display and error for precompile error ([#777](https://github.com/bluealloy/revm/pull/777))
- reorder env members ([#771](https://github.com/bluealloy/revm/pull/771))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v1.3.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>
* 83d27b0 - fix: use u128 for calc_blob_gasprice (#764) (16 hours ago) <Dan Cline>
* 1f86e45 - chore(deps): bump proptest from 1.2.0 to 1.3.1 (#763) (21 hours ago) <dependabot[bot]

# v1.2.0
date 28.09.2023

Summary of biggest changes:
* Some check for Env validity moved from revm to primitives crate.
* Cancun spec introduced.
* no_std added to primitives.
* introduce initcode size limit check taking config into account.
* deprecate `RefDBWrapper` for more generic `WrapDatabaseRef`.
* Implement `Error` for EVMError.
* Removal of hash from Bytecode.
* ChainId converted from U256 to u64.
* CfgEnv marked as `non_exhaustive` to accommodate future changes.
* Introduce `InvalidHeader` error that contains `prevrandao` and `blob gas` not set errors.
* c-kzg added as dependency as it is needed for `KzgSetting` that is sed inside EnvCfg.

Full git log:
* ea0d8d8 - fix: use u128 for calc data fee result (#757) (49 minutes ago) <Dan Cline>
* 4f916be - chore: bump c-kzg to create lib (#758) (5 hours ago) <rakita>
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 8a85d19 - fix: balance check disabled (#751) (25 hours ago) <Wodann>
* 8206193 - feat: add "kzg" as a separate feature (#746) (80 minutes ago) <DaniPopes>
* 26af13e - EIP-7516: BLOBBASEFEE opcode (#721) (5 days ago) <rakita>
* f72eaa0 - chore: error type for block header (#731) (5 days ago) <hack3r-0m>
* 1f31756 - document when InvalidTransaction errors are thrown (#722) (6 days ago) <Alessandro Mazza>
* cb39117 - fix(eip4844): Pass eth tests, additional conditions added. (#735) (6 days ago) <rakita>
* 70cf969 - chore: rm nonexhaustive for error (#726) (8 days ago) <Matthias Seitz>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* d615514 - chore: clippy incorrect_clone_impl_on_copy_type (#720) (12 days ago) <rakita>
* 5d68dd5 - chore(deps): bump bytes from 1.4.0 to 1.5.0 (#707) (2 weeks ago) <dependabot[bot]>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 616cc7e - chore(cfg): convert chain_id from u256 to u64 (#693) (3 weeks ago) <Lorenzo Feroleto>
* 7e7cb02 - Small doc comment fix (#698) (3 weeks ago) <refcell.eth>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* b0ee6d4 - feat: derive PartialEq, Eq for Env (#689) (3 weeks ago) <Dan Cline>
* 86d25c6 - chore: remove unused new_raw_with_hash (#676) (4 weeks ago) <Dan Cline>
* 175aaec - Removed the last dependencies breaking no-std build. (#669) (4 weeks ago) <Lucas Clemente Vella>
* f2929ad - chore(deps): bump proptest-derive from 0.3.0 to 0.4.0 (#652) (4 weeks ago) <dependabot[bot]>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 43d535c - style: bundle state (#637) (5 weeks ago) <Roman Krasiuk>
* 321152a - book workflow (#537) (5 weeks ago) <Waylon Jepsen>
* 0028193 - feat: Optional coinbase tip (#625) (5 weeks ago) <clabby>
* 3907fdf - chore: mark CfgEnv as non_exhaustive (#623) (5 weeks ago) <rakita>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* 449d6b9 - chore: export some `unreachable_pub` items (#598) (6 weeks ago) <DaniPopes>
* fc2107c - chore: Revert test, not change storage check , renaming of original slot value (#601) (6 weeks ago) <rakita>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* 5cdaa97 - chore: avoid unnecessary allocations (#581) (6 weeks ago) <DaniPopes>
* e9b6859 - chore(deps): bump bitflags from 2.3.3 to 2.4.0 (#596) (6 weeks ago) <dependabot[bot]>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 157ef36 - feat: introduce initcode size limit check taking config into account (#587) (7 weeks ago) <evalir>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* 781c8cc - feat: Implement `Error` for EVMError (#559) (9 weeks ago) <Yiannis Marangos>
* 5ce9dc9 - chore: clippy and fmt (#568) (9 weeks ago) <rakita>
* c153428 - feat(cancun): EIP-5656: MCOPY - Memory copying instruction (#528) (3 months ago) <Waylon Jepsen>
* 1839b3f - chore(deps): bump hashbrown from 0.13.2 to 0.14.0 (#519) (3 months ago) <dependabot[bot]>
* 63f9460 - chore(deps): bump auto_impl from 1.0.1 to 1.1.0 (#478) (3 months ago) <dependabot[bot]>
* b224874 - chore: add util functions for getting output data (#509) (4 months ago) <Matthias Seitz>
* e0ec1cc - chore: fix typo (#488) (4 months ago) <adria0.eth>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* d193418 - chore: Bundle inspector crate/call calls (#480) (5 months ago) <rakita>
* 75a6136 - feat: Introduce account status as bitflag inside JournalState (#477) (5 months ago) <rakita>


# v1.1.2
date: 03.05.2023

small release:
* ccefbca - chore(deps): bump ruint from 1.7.0 to 1.8.0 (#465) (50 minutes ago) <dependabot[bot]>
* d7adfd5 - Fix typo in primitives/src/state.rs (#474) (50 minutes ago) <Udoagwa Franklin>
* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>
# v1.1.1
date: 14.04.2023

One change:
* 4915bd1 - chore: add into_logs (#453) (3 days ago)


# v1.1.0
date: 04.04.2023

Mosty utility functions, additional checks and convenience changes.
Old bytecode that supported gas block was replaced with jumpmap only bitvec.

Changelog: 
* 992a11c - (HEAD -> v/310, origin/lib_versions) bump all (81 minutes ago) <rakita>
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 2d5b710 - Comment Fix (#430) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* 3d8ca66 - feat: add Output::into_data (#420) (3 weeks ago) <Matthias Seitz>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* cff1070 - Update readmdoc of `perf_analyse_created_bytecodes` (#404) (4 weeks ago) <rakita>
* 7bb73da - feat: Add check for chainID (#393) (4 weeks ago) <chirag-bgh>
* 3a17ca8 - feat: add b256<->u256 from impls (#398) (4 weeks ago) <Matthias Seitz>
* 3789509 - feat: add API to retrieve unpadded bytecode (#397) (5 weeks ago) <Wodann>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* 5efd9d1 - impl NonceTooHigh/ NonceTooLow checks (#383) (6 weeks ago) <gd>
* 188dacf - improvement: derive Debug for DatabaseComponentError (#377) (7 weeks ago) <Wodann>
* 0401cfd - Add B160/B256 From primitive_types traits (#380) (7 weeks ago) <Francesco Cinà>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* 5788340 - chore(deps): bump bytes from 1.3.0 to 1.4.0 (#355) (7 weeks ago) <dependabot[bot]>
* b4c62e9 - chore: rename Then to Than (#368) (7 weeks ago) <Matthias Seitz>
* 1c3e9e3 - improvement: use alloc & core for Arc impl (#367) (8 weeks ago) <Wodann>
* 3158ce9 - feat: implement Debug for DatabaseComponentError if supported (#363) (8 weeks ago) <Wodann>


* d9727c2 - improvement: add error details to InvalidTransaction::LackOfFundForGasLimit (#364) (8 weeks ago) <Wodann>
* 5d6ecd0 - improvement: implement BlockHash for Arc<BlockHashRef> (#361) (8 weeks ago) <Wodann>
* ae9baba - improvement: implement State for Arc<StateRef> (#360) (8 weeks ago) <Wodann>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>

# v1.0.0
date: 29.01.2023

Interpreter was extracted from main revm crate at the revm v3.0.0 version.