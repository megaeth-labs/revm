# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [41.0.0](https://github.com/megaeth-labs/revm/compare/revm-v40.0.3...revm-v41.0.0) - 2026-09-08

### Added

- add `OnStateHook` for `State<DB>` ([#3710](https://github.com/megaeth-labs/revm/pull/3710))
- *(precompile)* add aws-lc-rs as alternative backend for secp256r1 ([#3451](https://github.com/megaeth-labs/revm/pull/3451))
- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/megaeth-labs/revm/pull/3252))
- dynamically link gmp ([#3250](https://github.com/megaeth-labs/revm/pull/3250))
- allow EIP-7623 to be disabled ([#2985](https://github.com/megaeth-labs/revm/pull/2985))
- [**breaking**] Remove kzg-rs ([#2909](https://github.com/megaeth-labs/revm/pull/2909))
- Reuse bls12-381 codepaths to implement kzg point evaluation precompile ([#2809](https://github.com/megaeth-labs/revm/pull/2809))
- refactor test utils ([#2813](https://github.com/megaeth-labs/revm/pull/2813))
- add system transaction inspection support ([#2808](https://github.com/megaeth-labs/revm/pull/2808))
- expose sha3-keccak in revm and revm-primitives ([#2713](https://github.com/megaeth-labs/revm/pull/2713))
- optional_eip3541 ([#2661](https://github.com/megaeth-labs/revm/pull/2661))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/megaeth-labs/revm/pull/2596))
- enable P256 in Osaka ([#2601](https://github.com/megaeth-labs/revm/pull/2601))
- transact multi tx ([#2517](https://github.com/megaeth-labs/revm/pull/2517))
- support for system calls ([#2350](https://github.com/megaeth-labs/revm/pull/2350))
- Reorder revm deps in Cargo.toml ([#2331](https://github.com/megaeth-labs/revm/pull/2331))
- Add JournalInner ([#2311](https://github.com/megaeth-labs/revm/pull/2311))
- Add a wrapper for arkworks for EIP196 ([#2305](https://github.com/megaeth-labs/revm/pull/2305))
- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/megaeth-labs/revm/pull/2218))
- remove specification crate ([#2165](https://github.com/megaeth-labs/revm/pull/2165))
- book structure ([#2082](https://github.com/megaeth-labs/revm/pull/2082))
- Split Inspector trait from EthHandler into standalone crate ([#2075](https://github.com/megaeth-labs/revm/pull/2075))
- integrate alloy-eips ([#2078](https://github.com/megaeth-labs/revm/pull/2078))
- Evm structure (Cached Instructions and Precompiles) ([#2049](https://github.com/megaeth-labs/revm/pull/2049))
- Context execution ([#2013](https://github.com/megaeth-labs/revm/pull/2013))
- EthHandler trait ([#2001](https://github.com/megaeth-labs/revm/pull/2001))
- *(EIP-7623)* adjuct floor gas check order (main) ([#1991](https://github.com/megaeth-labs/revm/pull/1991))
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 ([#1965](https://github.com/megaeth-labs/revm/pull/1965))
- simplify Transaction trait ([#1959](https://github.com/megaeth-labs/revm/pull/1959))
- expose precompile address in Journal, DB::Error: StdError ([#1956](https://github.com/megaeth-labs/revm/pull/1956))
- integrate codspeed ([#1935](https://github.com/megaeth-labs/revm/pull/1935))
- Make Ctx journal generic ([#1933](https://github.com/megaeth-labs/revm/pull/1933))
- Restucturing Part7 Handler and Context rework ([#1865](https://github.com/megaeth-labs/revm/pull/1865))
- restructuring Part6 transaction crate ([#1814](https://github.com/megaeth-labs/revm/pull/1814))
- push NonceChange to Journal in deduct_caller ([#1804](https://github.com/megaeth-labs/revm/pull/1804))
- Merge validation/analyzis with Bytecode ([#1793](https://github.com/megaeth-labs/revm/pull/1793))
- Restructuring Part3 inspector crate ([#1788](https://github.com/megaeth-labs/revm/pull/1788))
- restructure Part2 database crate ([#1784](https://github.com/megaeth-labs/revm/pull/1784))
- project restructuring Part1 ([#1776](https://github.com/megaeth-labs/revm/pull/1776))
- to_plain_state ([#1778](https://github.com/megaeth-labs/revm/pull/1778))
- *(example)* deploy bytecode from scratch ([#1767](https://github.com/megaeth-labs/revm/pull/1767))
- introducing EvmWiring, a chain-specific configuration ([#1672](https://github.com/megaeth-labs/revm/pull/1672))
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/megaeth-labs/revm/pull/1695))
- check for typos in CI ([#1686](https://github.com/megaeth-labs/revm/pull/1686))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/megaeth-labs/revm/pull/1648))
- pass interpreter into Inspector::log ([#1610](https://github.com/megaeth-labs/revm/pull/1610))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/megaeth-labs/revm/pull/1607))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/megaeth-labs/revm/pull/1558))
- add bytecode_address from CallInputs to Contract during construction. ([#1568](https://github.com/megaeth-labs/revm/pull/1568))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/megaeth-labs/revm/pull/1565))
- *(EOF)* disallow ExtDelegateCall to legacy bytecode ([#1572](https://github.com/megaeth-labs/revm/pull/1572))
- *(EOF)* Add target address expansion checks ([#1570](https://github.com/megaeth-labs/revm/pull/1570))
- *(revm)* derive serde for `BundleState` ([#1539](https://github.com/megaeth-labs/revm/pull/1539))
- bump alloy, re-enable alloydb ([#1533](https://github.com/megaeth-labs/revm/pull/1533))
- mutable access for all fields in BundleBuilder ([#1524](https://github.com/megaeth-labs/revm/pull/1524))
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/megaeth-labs/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/megaeth-labs/revm/pull/1504))
- add helpers for working with instruction tables ([#1493](https://github.com/megaeth-labs/revm/pull/1493))
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/megaeth-labs/revm/pull/1499))
- Persist reverted account and storage slot lookups in `JournaledState` ([#1437](https://github.com/megaeth-labs/revm/pull/1437))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/megaeth-labs/revm/pull/1467))
- *(EOF)* Add EOF to inspector handle register ([#1469](https://github.com/megaeth-labs/revm/pull/1469))
- *(optimism)* Implement new L1 cost function for Fjord ([#1420](https://github.com/megaeth-labs/revm/pull/1420))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/megaeth-labs/revm/pull/1436))
- *(revm)* revert EIP-2935 BLOCKHASH opcode changes ([#1450](https://github.com/megaeth-labs/revm/pull/1450))
- load account should return db error ([#1447](https://github.com/megaeth-labs/revm/pull/1447))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/megaeth-labs/revm/pull/1415))
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/megaeth-labs/revm/pull/1389))
- add a hook to execute individual frames ([#1369](https://github.com/megaeth-labs/revm/pull/1369))
- *(Handler)* Add ClearHandle ([#1368](https://github.com/megaeth-labs/revm/pull/1368))
- Add uniswap V2 WETH-USDC swap example ([#1353](https://github.com/megaeth-labs/revm/pull/1353))
- *(interpreter)* add helpers for spending all gas ([#1360](https://github.com/megaeth-labs/revm/pull/1360))
- add helper methods to CallInputs ([#1345](https://github.com/megaeth-labs/revm/pull/1345))
- *(revm)* make `FrameOrResult` serializable ([#1282](https://github.com/megaeth-labs/revm/pull/1282))
- add flag to force hashbrown usage ([#1284](https://github.com/megaeth-labs/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/megaeth-labs/revm/pull/1143))
- *(`db`)* Introduce `alloydb` ([#1257](https://github.com/megaeth-labs/revm/pull/1257))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/megaeth-labs/revm/pull/1243))
- *(interpreter)* test Host object-safety, allow `dyn Host` in instructions ([#1245](https://github.com/megaeth-labs/revm/pull/1245))
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/megaeth-labs/revm/pull/1234))
- add convert_boxed and insert_boxed for InstructionTable ([#1194](https://github.com/megaeth-labs/revm/pull/1194))
- optional nonce check ([#1195](https://github.com/megaeth-labs/revm/pull/1195))
- Restrict ContextPrecompiles only to EvmContext ([#1174](https://github.com/megaeth-labs/revm/pull/1174))
- add insert method on instruction table ([#1167](https://github.com/megaeth-labs/revm/pull/1167))
- precompile with generic context ([#1155](https://github.com/megaeth-labs/revm/pull/1155))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/megaeth-labs/revm/pull/1157))
- add more JournaledState methods to EvmContext ([#1158](https://github.com/megaeth-labs/revm/pull/1158))
- add example for using a database by reference ([#1150](https://github.com/megaeth-labs/revm/pull/1150))
- Add boxed precompile trait ([#1131](https://github.com/megaeth-labs/revm/pull/1131))
- add with_handler method to EvmBuilder ([#1124](https://github.com/megaeth-labs/revm/pull/1124))
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/megaeth-labs/revm/pull/1106))
- split off serde_json dependency to its own feature ([#1104](https://github.com/megaeth-labs/revm/pull/1104))
- improve OriginalValuesKnown docs ([#1083](https://github.com/megaeth-labs/revm/pull/1083))
- *(handler)* Change spec id on &mut ([#1055](https://github.com/megaeth-labs/revm/pull/1055))
- *(Handler)* add push and pop of hanler registers ([#1053](https://github.com/megaeth-labs/revm/pull/1053))
- tweeks for v4.0 revm release ([#1048](https://github.com/megaeth-labs/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/megaeth-labs/revm/pull/1009))
- *(inspector)* Share call/create inputs in Inspector call_end/create_end ([#1003](https://github.com/megaeth-labs/revm/pull/1003))
- Convert optimism panic into graceful error ([#982](https://github.com/megaeth-labs/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/megaeth-labs/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/megaeth-labs/revm/pull/972))
- *(ethersdb)* propagate errors instead of panicking in basic_ref ([#935](https://github.com/megaeth-labs/revm/pull/935))
- *(revm)* implement prepend_state for BundleState ([#907](https://github.com/megaeth-labs/revm/pull/907))
- add serde derives for `CacheDB` under "serde" flag ([#911](https://github.com/megaeth-labs/revm/pull/911))
- *(examples)* generate block traces ([#895](https://github.com/megaeth-labs/revm/pull/895))
- *(revm)* Evm Context Tests and test-utils Feature ([#903](https://github.com/megaeth-labs/revm/pull/903))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/megaeth-labs/revm/pull/871))
- Loop call stack ([#851](https://github.com/megaeth-labs/revm/pull/851))
- transition account balance delta ([#843](https://github.com/megaeth-labs/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/megaeth-labs/revm/pull/834))
- add more `auto_impl`s to revm traits ([#799](https://github.com/megaeth-labs/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/megaeth-labs/revm/pull/794))
- derive more traits ([#745](https://github.com/megaeth-labs/revm/pull/745))
- add methods to `CreateInput` for calculating created address ([#793](https://github.com/megaeth-labs/revm/pull/793))
- *(revm)* implement DatabaseRef trait for EthersDB ([#774](https://github.com/megaeth-labs/revm/pull/774))
- Alloy primitives ([#724](https://github.com/megaeth-labs/revm/pull/724))
- Optimism execution changes ([#682](https://github.com/megaeth-labs/revm/pull/682))
- add "kzg" as a separate feature ([#746](https://github.com/megaeth-labs/revm/pull/746))
- implement EIP-4844 ([#668](https://github.com/megaeth-labs/revm/pull/668))
- *(state)* remove state sorting, no_std ci,remove rayon ([#717](https://github.com/megaeth-labs/revm/pull/717))
- return wiped inside storage changeset ([#711](https://github.com/megaeth-labs/revm/pull/711))
- *(state)* Nits, builder option and OriginalValueKnown flags ([#699](https://github.com/megaeth-labs/revm/pull/699))
- *(StateBuilder)* switch builder option from without_bundle to with_bundle ([#688](https://github.com/megaeth-labs/revm/pull/688))
- *(state)* take N reverts from BundleState, struct refactor ([#681](https://github.com/megaeth-labs/revm/pull/681))
- add BundleState::revert_latest ([#661](https://github.com/megaeth-labs/revm/pull/661))
- *(state)* add a flag allowing transition merge without reverts ([#657](https://github.com/megaeth-labs/revm/pull/657))
- *(state)* Make Bundle extend wipe aware ([#655](https://github.com/megaeth-labs/revm/pull/655))
- *(state)* ability to disable reverts collection in bundle state ([#654](https://github.com/megaeth-labs/revm/pull/654))
- *(`interpreter`)* add hash to bytecode ([#628](https://github.com/megaeth-labs/revm/pull/628))
- Optional coinbase tip ([#625](https://github.com/megaeth-labs/revm/pull/625))
- *(state)* Use preloaded bundle inside state ([#622](https://github.com/megaeth-labs/revm/pull/622))
- *(state)* Block hash cache and overrides ([#621](https://github.com/megaeth-labs/revm/pull/621))
- alloy migration ([#535](https://github.com/megaeth-labs/revm/pull/535))
- State with account status ([#499](https://github.com/megaeth-labs/revm/pull/499))
- EIP-1153 Transient storage opcodes ([#546](https://github.com/megaeth-labs/revm/pull/546))
- separate initial checks ([#486](https://github.com/megaeth-labs/revm/pull/486))
- Create account checkpoint ([#483](https://github.com/megaeth-labs/revm/pull/483))
- Introduce account status as bitflag inside JournalState ([#477](https://github.com/megaeth-labs/revm/pull/477))
- Add all internals results to Halt ([#413](https://github.com/megaeth-labs/revm/pull/413))
- add contract+target to selfdestruct hook ([#410](https://github.com/megaeth-labs/revm/pull/410))
- Add check for chainID ([#393](https://github.com/megaeth-labs/revm/pull/393))
- add EVM::with_env ([#385](https://github.com/megaeth-labs/revm/pull/385))
- json opcode traces EIP-3155 ([#356](https://github.com/megaeth-labs/revm/pull/356))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/megaeth-labs/revm/pull/376))
- Different OutOfGas Error types ([#354](https://github.com/megaeth-labs/revm/pull/354))
- revm-interpreter created ([#320](https://github.com/megaeth-labs/revm/pull/320))
- allow disabling of balance checks ([#297](https://github.com/megaeth-labs/revm/pull/297))
- Export CustomPrinter insector from revm ([#300](https://github.com/megaeth-labs/revm/pull/300))
- substitute web3db to ethersdb ([#293](https://github.com/megaeth-labs/revm/pull/293))
- *(revm)* Return `bytes` in Create calls ([#289](https://github.com/megaeth-labs/revm/pull/289))
- *(interpreter)* Unify instruction fn signature ([#283](https://github.com/megaeth-labs/revm/pull/283))
- *(revm)* Add prevrandao field to EnvBlock ([#271](https://github.com/megaeth-labs/revm/pull/271))
- *(refactor)* make keccak in one place. ([#247](https://github.com/megaeth-labs/revm/pull/247))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/megaeth-labs/revm/pull/239))
- allow block gas limit to be toggled off ([#238](https://github.com/megaeth-labs/revm/pull/238))
- allow eip3607 to be toggled off ([#237](https://github.com/megaeth-labs/revm/pull/237))
- *(revm, revme)* gas inspector ([#222](https://github.com/megaeth-labs/revm/pull/222))
- add Memory::shrink_to_fit ([#215](https://github.com/megaeth-labs/revm/pull/215))
- expose hash on `BytecodeLocked` ([#189](https://github.com/megaeth-labs/revm/pull/189))
- *(revm)* more default trait implementations ([#181](https://github.com/megaeth-labs/revm/pull/181))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/megaeth-labs/revm/pull/156))
- add Subroutine debug clone derive ([#128](https://github.com/megaeth-labs/revm/pull/128))
- add ord derives to specid ([#127](https://github.com/megaeth-labs/revm/pull/127))
- add getters for cachedb ([#119](https://github.com/megaeth-labs/revm/pull/119))
- add serde support to model types ([#91](https://github.com/megaeth-labs/revm/pull/91))
- add some PartialEq derives ([#90](https://github.com/megaeth-labs/revm/pull/90))
- `Inspector::log` ([#85](https://github.com/megaeth-labs/revm/pull/85))
- mutable call inputs
- call insp end functions on early return
- cache block hashes ([#71](https://github.com/megaeth-labs/revm/pull/71))
- implement `DatabaseRef` for `CacheDB`

### Fixed

- expose optional_priority_fee_check feature from context crate ([#3159](https://github.com/megaeth-labs/revm/pull/3159))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/megaeth-labs/revm/pull/2978))
- features and check in ci ([#2766](https://github.com/megaeth-labs/revm/pull/2766))
- gas deduction with `disable_balance_check` ([#2699](https://github.com/megaeth-labs/revm/pull/2699))
- call stack_frame.clear() at end ([#2656](https://github.com/megaeth-labs/revm/pull/2656))
- compare_or_save_testdata sig ([#2637](https://github.com/megaeth-labs/revm/pull/2637))
- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/megaeth-labs/revm/pull/2581))
- fix typo and update links ([#2387](https://github.com/megaeth-labs/revm/pull/2387))
- correct propagate features ([#2177](https://github.com/megaeth-labs/revm/pull/2177))
- *(precompiles)* add portable flag for bls ([#2174](https://github.com/megaeth-labs/revm/pull/2174))
- make macro crate-agnostic ([#1802](https://github.com/megaeth-labs/revm/pull/1802))
- *(inspector)* always call selfdestruct if entry is made ([#1746](https://github.com/megaeth-labs/revm/pull/1746))
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/megaeth-labs/revm/pull/1605))
- correctly calculate eofcreate address ([#1619](https://github.com/megaeth-labs/revm/pull/1619))
- allow non-static lifetime in HandleRegisterBox ([#1608](https://github.com/megaeth-labs/revm/pull/1608))
- *(EOF)* Use cfg code size limit for eofcreate ([#1606](https://github.com/megaeth-labs/revm/pull/1606))
- *(eof)* fixture 2 tests ([#1550](https://github.com/megaeth-labs/revm/pull/1550))
- *(eof)* output gas for eofcreate ([#1540](https://github.com/megaeth-labs/revm/pull/1540))
- *(revm)* remove storage reset that clears is_cold flag ([#1518](https://github.com/megaeth-labs/revm/pull/1518))
- *(op)* Remove `U256::from(<float>)` ([#1498](https://github.com/megaeth-labs/revm/pull/1498))
- *(EOF)* panic on empty input range, and continue exec after eofcreate ([#1477](https://github.com/megaeth-labs/revm/pull/1477))
- *(Interpreter)* wrong block number used ([#1458](https://github.com/megaeth-labs/revm/pull/1458))
- blockchash for devnet-0  ([#1427](https://github.com/megaeth-labs/revm/pull/1427))
- *(eip2935)* Preload blockchash storage address ([#1395](https://github.com/megaeth-labs/revm/pull/1395))
- return the correct error in resize_memory ([#1359](https://github.com/megaeth-labs/revm/pull/1359))
- *(TracerEip3155)* clear Inspector data after transaction. ([#1230](https://github.com/megaeth-labs/revm/pull/1230))
- *(GasInspector)* calculate correct remaining gas after call return ([#1236](https://github.com/megaeth-labs/revm/pull/1236))
- fix eip3155 summary gas_used bug and add fork name ([#1216](https://github.com/megaeth-labs/revm/pull/1216))
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/megaeth-labs/revm/pull/1169))
- gas cost calculation ([#1166](https://github.com/megaeth-labs/revm/pull/1166))
- reset tstorage on finalize ([#1168](https://github.com/megaeth-labs/revm/pull/1168))
- make `feature = "optional_gas_refund"` work ([#1134](https://github.com/megaeth-labs/revm/pull/1134))
- replace tuple in sstore return with struct ([#1115](https://github.com/megaeth-labs/revm/pull/1115))
- fix EthersDB deadlock ([#1089](https://github.com/megaeth-labs/revm/pull/1089))
- Handle fatal db error on load_account ([#1111](https://github.com/megaeth-labs/revm/pull/1111))
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/megaeth-labs/revm/pull/1098))
- modify cfg spec_id ([#1095](https://github.com/megaeth-labs/revm/pull/1095)) ([#1096](https://github.com/megaeth-labs/revm/pull/1096))
- optimism compilation ([#1091](https://github.com/megaeth-labs/revm/pull/1091))
- properly set context env ([#1070](https://github.com/megaeth-labs/revm/pull/1070))
- typo on internal append_handle_register methods ([#1069](https://github.com/megaeth-labs/revm/pull/1069))
- *(op)* skip validation on deposit tx ([#1065](https://github.com/megaeth-labs/revm/pull/1065))
- fix previous commit ([#1044](https://github.com/megaeth-labs/revm/pull/1044))
- *(State)* Preserve original values on delete revert ([#1010](https://github.com/megaeth-labs/revm/pull/1010))
- optimism gas refunds ([#989](https://github.com/megaeth-labs/revm/pull/989))
- dont calculate initcode keccak on CREATE ([#969](https://github.com/megaeth-labs/revm/pull/969))
- *(ci)* Workflow Touchups ([#901](https://github.com/megaeth-labs/revm/pull/901))
- safer stack ([#879](https://github.com/megaeth-labs/revm/pull/879))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/megaeth-labs/revm/pull/824))
- fix typos in revm crate ([#821](https://github.com/megaeth-labs/revm/pull/821))
- Optimism execution ([#789](https://github.com/megaeth-labs/revm/pull/789))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/megaeth-labs/revm/pull/795))
- use u128 for calc data fee result ([#757](https://github.com/megaeth-labs/revm/pull/757))
- *(eip4844)* Pass eth tests, additional conditions added. ([#735](https://github.com/megaeth-labs/revm/pull/735))
- use CANCUN precompile id for CANCUN SpecId ([#733](https://github.com/megaeth-labs/revm/pull/733))
- *(state)* Extend now properly transfers wiped storage ([#675](https://github.com/megaeth-labs/revm/pull/675))
- *(state)* retain destroyed account status on bundle extend ([#667](https://github.com/megaeth-labs/revm/pull/667))
- *(state)* Regresion, remove present info on selfdestruct ([#664](https://github.com/megaeth-labs/revm/pull/664))
- *(state)* state transition regression ([#662](https://github.com/megaeth-labs/revm/pull/662))
- *(state)* drop storage only for DestroyedChanged ([#651](https://github.com/megaeth-labs/revm/pull/651))
- fix revert from DestroyedChanged to DestroyedAgain ([#648](https://github.com/megaeth-labs/revm/pull/648))
- *(state)* check if storage revert is empty ([#643](https://github.com/megaeth-labs/revm/pull/643))
- *(state)* return RevertToSlot struct with more info ([#636](https://github.com/megaeth-labs/revm/pull/636))
- fix typos ([#620](https://github.com/megaeth-labs/revm/pull/620))
- *(inspector)* call call_end/create_end when inspector shortcircuits calls ([#609](https://github.com/megaeth-labs/revm/pull/609))
- Load caller in safe way in finalization fn ([#604](https://github.com/megaeth-labs/revm/pull/604))
- *(transient_storage)* set previous value in journal ([#585](https://github.com/megaeth-labs/revm/pull/585))
- AccessList with two same addresses ([#578](https://github.com/megaeth-labs/revm/pull/578))
- *(revm)* EIP-3155 tracer tx output without debug artefact ([#552](https://github.com/megaeth-labs/revm/pull/552))
- *(revm)* extra return in EIP3155 inspector ([#563](https://github.com/megaeth-labs/revm/pull/563))
- *(revm)* include CREATE/CREATE2 in EIP3155 inspector ([#562](https://github.com/megaeth-labs/revm/pull/562))
- *(doc)* Inline documentation of re-exports ([#560](https://github.com/megaeth-labs/revm/pull/560))
- fix comment ([#529](https://github.com/megaeth-labs/revm/pull/529))
- typo in eip-3155 output ([#497](https://github.com/megaeth-labs/revm/pull/497))
- revert of selfdestruct with same target address ([#475](https://github.com/megaeth-labs/revm/pull/475))
- compile errors for features ([#467](https://github.com/megaeth-labs/revm/pull/467))
- touched account on creation ([#463](https://github.com/megaeth-labs/revm/pull/463))
- k256 compile error ([#451](https://github.com/megaeth-labs/revm/pull/451))
- *(db)* preserve existing account state ([#414](https://github.com/megaeth-labs/revm/pull/414))
- call create_end for all code paths ([#362](https://github.com/megaeth-labs/revm/pull/362))
- disable balance check ([#342](https://github.com/megaeth-labs/revm/pull/342))
- feature flags ([#330](https://github.com/megaeth-labs/revm/pull/330))
- broken feature flags ([#319](https://github.com/megaeth-labs/revm/pull/319))
- feature flag compiler errors ([#256](https://github.com/megaeth-labs/revm/pull/256))
- fix web3db sanity check ([#245](https://github.com/megaeth-labs/revm/pull/245))
- return out of gas code for precompiled contracts ([#234](https://github.com/megaeth-labs/revm/pull/234))
- make DatabaseRef::basic consistent with Database ([#201](https://github.com/megaeth-labs/revm/pull/201))
- Use `saturating_add` instead of `checked_add` in `finalize` ([#184](https://github.com/megaeth-labs/revm/pull/184))
- *(revm)* Fix balance overflow in `finalize` ([#182](https://github.com/megaeth-labs/revm/pull/182))
- set gas_block to empty bytecode ([#172](https://github.com/megaeth-labs/revm/pull/172))
- BLOCKHASH should return 0 if number not in last 256 blocks ([#112](https://github.com/megaeth-labs/revm/pull/112))
- impose a memory limit ([#86](https://github.com/megaeth-labs/revm/pull/86))
- interpreter gas should immutably borrow self
- return spent gas with refunds accounted for
- various inspector fixes ([#69](https://github.com/megaeth-labs/revm/pull/69))
- call inspector `step` and `step_end`
- export missing machine structs
- export `Filth`
- make `*_ref` functions take `&self`
- *(clippy)* fix some clippy lints

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
- bump revm-database-interface to v10.0.0 and all dependents (v107) ([#3474](https://github.com/megaeth-labs/revm/pull/3474))
- bump revm to v35.0.0 and update changelogs for v103/v104 ([#3470](https://github.com/megaeth-labs/revm/pull/3470))
- release ([#3316](https://github.com/megaeth-labs/revm/pull/3316))
- remove GPL mention and update gmp feature comments ([#3383](https://github.com/megaeth-labs/revm/pull/3383))
- revm v34.0.0 ([#3313](https://github.com/megaeth-labs/revm/pull/3313))
- release ([#3175](https://github.com/megaeth-labs/revm/pull/3175))
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
- consolidate revme imports ([#3088](https://github.com/megaeth-labs/revm/pull/3088))
- tag v93 revm v30.1.0 ([#3112](https://github.com/megaeth-labs/revm/pull/3112))
- release ([#3108](https://github.com/megaeth-labs/revm/pull/3108))
- release ([#3102](https://github.com/megaeth-labs/revm/pull/3102))
- release ([#3079](https://github.com/megaeth-labs/revm/pull/3079))
- bump minor versions ([#3078](https://github.com/megaeth-labs/revm/pull/3078))
- release ([#3061](https://github.com/megaeth-labs/revm/pull/3061))
- release ([#2958](https://github.com/megaeth-labs/revm/pull/2958))
- changelog update for v87 ([#3056](https://github.com/megaeth-labs/revm/pull/3056))
- add boundless ([#3043](https://github.com/megaeth-labs/revm/pull/3043))
- forward optional_fee_charge feature ([#3005](https://github.com/megaeth-labs/revm/pull/3005))
- add SECURITY.md ([#2956](https://github.com/megaeth-labs/revm/pull/2956))
- tag v86 revm v29.0.0 ([#2912](https://github.com/megaeth-labs/revm/pull/2912))
- release ([#2899](https://github.com/megaeth-labs/revm/pull/2899))
- release ([#2873](https://github.com/megaeth-labs/revm/pull/2873))
- rm ee-test from revm/op-revm ([#2857](https://github.com/megaeth-labs/revm/pull/2857))
- tag v84 revm v28.0.0 ([#2856](https://github.com/megaeth-labs/revm/pull/2856))
- release ([#2854](https://github.com/megaeth-labs/revm/pull/2854))
- update README.md ([#2842](https://github.com/megaeth-labs/revm/pull/2842))
- reuse global crypto provide idea ([#2786](https://github.com/megaeth-labs/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/megaeth-labs/revm/pull/2789))
- release ([#2771](https://github.com/megaeth-labs/revm/pull/2771))
- add asm-sha2 feature for sha2 precompile ([#2712](https://github.com/megaeth-labs/revm/pull/2712))
- release ([#2682](https://github.com/megaeth-labs/revm/pull/2682))
- tag v81 revm v27.0.1 ([#2689](https://github.com/megaeth-labs/revm/pull/2689))
- v80 revm v27.0.1 ([#2683](https://github.com/megaeth-labs/revm/pull/2683))
- tag v79 revm v27.0.0 ([#2680](https://github.com/megaeth-labs/revm/pull/2680))
- release ([#2659](https://github.com/megaeth-labs/revm/pull/2659))
- cargo clippy --fix --all ([#2671](https://github.com/megaeth-labs/revm/pull/2671))
- inline documentation of revm top modules ([#2666](https://github.com/megaeth-labs/revm/pull/2666))
- use TxEnv::builder ([#2652](https://github.com/megaeth-labs/revm/pull/2652))
- release ([#2657](https://github.com/megaeth-labs/revm/pull/2657))
- release ([#2641](https://github.com/megaeth-labs/revm/pull/2641))
- lints for examples ([#2650](https://github.com/megaeth-labs/revm/pull/2650))
- rename `transact` methods ([#2616](https://github.com/megaeth-labs/revm/pull/2616))
- tag v76 revm v25.0.0 ([#2590](https://github.com/megaeth-labs/revm/pull/2590))
- release ([#2577](https://github.com/megaeth-labs/revm/pull/2577))
- tag v75 revm v24.0.1 ([#2563](https://github.com/megaeth-labs/revm/pull/2563)) ([#2589](https://github.com/megaeth-labs/revm/pull/2589))
- tag v74 revm v24.0.0 ([#2539](https://github.com/megaeth-labs/revm/pull/2539))
- release ([#2527](https://github.com/megaeth-labs/revm/pull/2527))
- make crates.io version badge clickable ([#2526](https://github.com/megaeth-labs/revm/pull/2526))
- tag v71, revm v23.1.0 semver major bump ([#2492](https://github.com/megaeth-labs/revm/pull/2492))
- changelog tag v70, revm v23.0.0 ([#2488](https://github.com/megaeth-labs/revm/pull/2488))
- release ([#2487](https://github.com/megaeth-labs/revm/pull/2487))
- typos ([#2474](https://github.com/megaeth-labs/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/megaeth-labs/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/megaeth-labs/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/megaeth-labs/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/megaeth-labs/revm/pull/2415)) ([#2416](https://github.com/megaeth-labs/revm/pull/2416))
- bump v68 revm v22.0.0 ([#2396](https://github.com/megaeth-labs/revm/pull/2396))
- tag v67 revm v21.0.0 ([#2341](https://github.com/megaeth-labs/revm/pull/2341))
- release-plz ([#2340](https://github.com/megaeth-labs/revm/pull/2340))
- Propagate asyncdb feature flag from database-interface to revm  ([#2310](https://github.com/megaeth-labs/revm/pull/2310))
- links to main readme ([#2298](https://github.com/megaeth-labs/revm/pull/2298))
- add links to arch page ([#2297](https://github.com/megaeth-labs/revm/pull/2297))
- revm v20.0.0 stable version, tag v66 ([#2294](https://github.com/megaeth-labs/revm/pull/2294))
- v65 revm: v20.0.0-alpha.7 ([#2280](https://github.com/megaeth-labs/revm/pull/2280))
- tag v63 revm v20.0.0-alpha.6 ([#2219](https://github.com/megaeth-labs/revm/pull/2219))
- tag v62 revm v20.0.0-alpha.5 ([#2198](https://github.com/megaeth-labs/revm/pull/2198))
- tag v61 revm v20.0.0-alpha.4 ([#2190](https://github.com/megaeth-labs/revm/pull/2190))
- tag v60, revm v20.0.0-alpha.3 ([#2176](https://github.com/megaeth-labs/revm/pull/2176))
- v59 release-plz update ([#2170](https://github.com/megaeth-labs/revm/pull/2170))
- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/megaeth-labs/revm/pull/2155))
- examples to use main revm crate ([#2152](https://github.com/megaeth-labs/revm/pull/2152))
- rename revm-optimism to op-revm ([#2141](https://github.com/megaeth-labs/revm/pull/2141))
- move mainnet builder to handler crate ([#2138](https://github.com/megaeth-labs/revm/pull/2138))
- fix README link ([#2139](https://github.com/megaeth-labs/revm/pull/2139))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/megaeth-labs/revm/pull/2132))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/megaeth-labs/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/megaeth-labs/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/megaeth-labs/revm/pull/2088))
- tag v57 revm 20.0.0-alpha.1 ([#2086](https://github.com/megaeth-labs/revm/pull/2086))
- Rename NameTrait to NameTr ([#2084](https://github.com/megaeth-labs/revm/pull/2084))
- backport op l1 fetch perf ([#2076](https://github.com/megaeth-labs/revm/pull/2076))
- API cleanup ([#2067](https://github.com/megaeth-labs/revm/pull/2067))
- Add helpers with_inspector with_precompile ([#2063](https://github.com/megaeth-labs/revm/pull/2063))
- Bump licence year to 2025 ([#2058](https://github.com/megaeth-labs/revm/pull/2058))
- bump alloy versions to match latest ([#2007](https://github.com/megaeth-labs/revm/pull/2007))
- align crates versions ([#1983](https://github.com/megaeth-labs/revm/pull/1983))
- Make inspector use generics, rm associated types ([#1934](https://github.com/megaeth-labs/revm/pull/1934))
- fix comments and docs into more sensible ([#1920](https://github.com/megaeth-labs/revm/pull/1920))
- EVM transact, make output generic for POSTEXEC ([#1931](https://github.com/megaeth-labs/revm/pull/1931))
- *(readme)* add tycho-simulation to "Used by" ([#1926](https://github.com/megaeth-labs/revm/pull/1926))
- Move CfgEnv from context-interface to context crate ([#1910](https://github.com/megaeth-labs/revm/pull/1910))
- bumps select alloy crates to 0.6 ([#1854](https://github.com/megaeth-labs/revm/pull/1854))
- Update README.md examples section ([#1853](https://github.com/megaeth-labs/revm/pull/1853))
- some no_std cleanup ([#1834](https://github.com/megaeth-labs/revm/pull/1834))
- bump alloy to 0.4.2 ([#1817](https://github.com/megaeth-labs/revm/pull/1817))
- *(primitives)* replace HashMap re-exports with alloy_primitives::map ([#1805](https://github.com/megaeth-labs/revm/pull/1805))
- *(deps)* bump anyhow from 1.0.88 to 1.0.89 ([#1772](https://github.com/megaeth-labs/revm/pull/1772))
- simplify SuccessOrHalt trait bound ([#1768](https://github.com/megaeth-labs/revm/pull/1768))
- *(deps)* bump alloy-sol-types from 0.8.0 to 0.8.2 ([#1762](https://github.com/megaeth-labs/revm/pull/1762))
- *(deps)* bump anyhow from 1.0.86 to 1.0.87 ([#1760](https://github.com/megaeth-labs/revm/pull/1760))
- make clippy happy ([#1755](https://github.com/megaeth-labs/revm/pull/1755))
- Test l1 gas used fjord ([#1749](https://github.com/megaeth-labs/revm/pull/1749))
- Add test for `revm::optimism::L1BlockInfo::calculate_tx_l1_cost_fjord` ([#1743](https://github.com/megaeth-labs/revm/pull/1743))
- *(deps)* bump tokio from 1.39.2 to 1.40.0 ([#1739](https://github.com/megaeth-labs/revm/pull/1739))
- release ([#1729](https://github.com/megaeth-labs/revm/pull/1729))
- Bump new logo ([#1735](https://github.com/megaeth-labs/revm/pull/1735))
- main changelog ([#1730](https://github.com/megaeth-labs/revm/pull/1730))
- release ([#1722](https://github.com/megaeth-labs/revm/pull/1722))
- *(deps)* bump alloy and primitives ([#1725](https://github.com/megaeth-labs/revm/pull/1725))
- cast block number to u64 and not usize ([#1727](https://github.com/megaeth-labs/revm/pull/1727))
- clean up some journalstate docs ([#1712](https://github.com/megaeth-labs/revm/pull/1712))
- update some docs related to state ([#1711](https://github.com/megaeth-labs/revm/pull/1711))
- tag v41 revm v13.0.0 ([#1692](https://github.com/megaeth-labs/revm/pull/1692))
- release ([#1683](https://github.com/megaeth-labs/revm/pull/1683))
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/megaeth-labs/revm/pull/1685))
- *(deps)* bump rstest from 0.21.0 to 0.22.0 ([#1681](https://github.com/megaeth-labs/revm/pull/1681))
- *(deps)* bump tokio from 1.38.1 to 1.39.2 ([#1668](https://github.com/megaeth-labs/revm/pull/1668))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/megaeth-labs/revm/pull/1661))
- avoid cloning original_bytes ([#1646](https://github.com/megaeth-labs/revm/pull/1646))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/megaeth-labs/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/megaeth-labs/revm/pull/1621))
- *(eof)* simplify magic checks ([#1633](https://github.com/megaeth-labs/revm/pull/1633))
- bump versions bcs of primitives ([#1631](https://github.com/megaeth-labs/revm/pull/1631))
- bump main changelog ([#1630](https://github.com/megaeth-labs/revm/pull/1630))
- *(EOF)* Use Bytecode::new_legacy ([#1628](https://github.com/megaeth-labs/revm/pull/1628))
- release ([#1620](https://github.com/megaeth-labs/revm/pull/1620))
- bump alloy deps ([#1623](https://github.com/megaeth-labs/revm/pull/1623))
- *(deps)* bump alloy-sol-types from 0.7.6 to 0.7.7 ([#1614](https://github.com/megaeth-labs/revm/pull/1614))
- group optimism invalid txn errors ([#1604](https://github.com/megaeth-labs/revm/pull/1604))
- load_account -> warm_preloaded_addresses ([#1584](https://github.com/megaeth-labs/revm/pull/1584))
- Refactor code, and check if precompile for create collision ([#1600](https://github.com/megaeth-labs/revm/pull/1600))
- *(revm)* defer bytecode load ([#1588](https://github.com/megaeth-labs/revm/pull/1588))
- Rename gas_price to gas_limit for precompile args ([#1593](https://github.com/megaeth-labs/revm/pull/1593))
- release ([#1579](https://github.com/megaeth-labs/revm/pull/1579))
- bump precompile to v9.0.0 ([#1590](https://github.com/megaeth-labs/revm/pull/1590))
- *(README)* add rbuilder to used-by ([#1585](https://github.com/megaeth-labs/revm/pull/1585))
- Use HandleOrRuntime to allow alloydb/ethersdb to hold a custom runtime ([#1576](https://github.com/megaeth-labs/revm/pull/1576))
- store tokio::runtime::Handle in ethers/alloyDB ([#1557](https://github.com/megaeth-labs/revm/pull/1557))
- use const blocks ([#1522](https://github.com/megaeth-labs/revm/pull/1522))
- fix compile for alloydb ([#1559](https://github.com/megaeth-labs/revm/pull/1559))
- replace AccessList with alloy version ([#1552](https://github.com/megaeth-labs/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/megaeth-labs/revm/pull/1505))
- release ([#1548](https://github.com/megaeth-labs/revm/pull/1548))
- Add CI build target for no-std + optimism, use matrix builds ([#1551](https://github.com/megaeth-labs/revm/pull/1551))
- replace TransactTo with TxKind ([#1542](https://github.com/megaeth-labs/revm/pull/1542))
- avoid cloning precompiles ([#1486](https://github.com/megaeth-labs/revm/pull/1486))
- add setters to `BundleBuilder` with `&mut self` ([#1527](https://github.com/megaeth-labs/revm/pull/1527))
- pluralize EOFCreateInput ([#1523](https://github.com/megaeth-labs/revm/pull/1523))
- added simular to used-by ([#1521](https://github.com/megaeth-labs/revm/pull/1521))
- Removed .clone() in ExecutionHandler::call, and reusing output buffer in Interpreter ([#1512](https://github.com/megaeth-labs/revm/pull/1512))
- remove old deprecated items ([#1489](https://github.com/megaeth-labs/revm/pull/1489))
- *(deps)* bump rstest from 0.19.0 to 0.21.0 ([#1482](https://github.com/megaeth-labs/revm/pull/1482))
- *(deps)* bump tokio from 1.37.0 to 1.38.0 ([#1480](https://github.com/megaeth-labs/revm/pull/1480))
- *(primitives)* rename State/Storage to EvmState/EvmStorage ([#1459](https://github.com/megaeth-labs/revm/pull/1459))
- remove 'checked' bytecode bench causing benchmarks to crash due to name ([#1461](https://github.com/megaeth-labs/revm/pull/1461))
- cargo update ([#1451](https://github.com/megaeth-labs/revm/pull/1451))
- cleanup host blockhash fn ([#1430](https://github.com/megaeth-labs/revm/pull/1430))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/megaeth-labs/revm/pull/1354))" ([#1424](https://github.com/megaeth-labs/revm/pull/1424))" ([#1426](https://github.com/megaeth-labs/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/megaeth-labs/revm/pull/1354))" ([#1424](https://github.com/megaeth-labs/revm/pull/1424))
- *(deps)* bump anyhow from 1.0.82 to 1.0.83 ([#1404](https://github.com/megaeth-labs/revm/pull/1404))
- remove alloydb example as the crate is not published ([#1398](https://github.com/megaeth-labs/revm/pull/1398))
- release ([#1261](https://github.com/megaeth-labs/revm/pull/1261))
- add Trin to used by list ([#1393](https://github.com/megaeth-labs/revm/pull/1393))
- refactor lints ([#1386](https://github.com/megaeth-labs/revm/pull/1386))
- bump alloy & specify dep rev ([#1380](https://github.com/megaeth-labs/revm/pull/1380))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/megaeth-labs/revm/pull/1374))
- *(ci)* bump action/deploy ([#1372](https://github.com/megaeth-labs/revm/pull/1372))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/megaeth-labs/revm/pull/1307))
- *(deps)* bump anyhow from 1.0.81 to 1.0.82 ([#1293](https://github.com/megaeth-labs/revm/pull/1293))
- fix some warnings ([#1305](https://github.com/megaeth-labs/revm/pull/1305))
- Update documentation ([#1275](https://github.com/megaeth-labs/revm/pull/1275))
- *(interpreter)* use `pop_top!` where possible ([#1267](https://github.com/megaeth-labs/revm/pull/1267))
- add and use EvmContext::take_error ([#1264](https://github.com/megaeth-labs/revm/pull/1264))
- release ([#1231](https://github.com/megaeth-labs/revm/pull/1231))
- use uint macro & fix various small things ([#1253](https://github.com/megaeth-labs/revm/pull/1253))
- *(deps)* bump tokio from 1.36.0 to 1.37.0 ([#1244](https://github.com/megaeth-labs/revm/pull/1244))
- *(interpreter)* unbox contract field ([#1228](https://github.com/megaeth-labs/revm/pull/1228))
- *(primitives)* kzg intro ([#1209](https://github.com/megaeth-labs/revm/pull/1209))
- *(interpreter)* keep track of remaining gas rather than spent ([#1221](https://github.com/megaeth-labs/revm/pull/1221))
- Improve `EthersDB` ([#1208](https://github.com/megaeth-labs/revm/pull/1208))
- Revert "feat: optional nonce check ([#1195](https://github.com/megaeth-labs/revm/pull/1195))" ([#1212](https://github.com/megaeth-labs/revm/pull/1212))
- release ([#1175](https://github.com/megaeth-labs/revm/pull/1175))
- Change unwrap to ? to propagate errors ([#1207](https://github.com/megaeth-labs/revm/pull/1207))
- fix wonky test ([#1197](https://github.com/megaeth-labs/revm/pull/1197))
- clippy ([#1196](https://github.com/megaeth-labs/revm/pull/1196))
- *(deps)* bump anyhow from 1.0.80 to 1.0.81 ([#1187](https://github.com/megaeth-labs/revm/pull/1187))
- fix some typos ([#1189](https://github.com/megaeth-labs/revm/pull/1189))
- Fix typo in readme ([#1185](https://github.com/megaeth-labs/revm/pull/1185))
- Update post_execution.rs ([#1180](https://github.com/megaeth-labs/revm/pull/1180))
- tag v32 revm v7.1.0 ([#1176](https://github.com/megaeth-labs/revm/pull/1176))
- release ([#1125](https://github.com/megaeth-labs/revm/pull/1125))
- *(deps)* bump ethers-contract from 2.0.13 to 2.0.14 ([#1161](https://github.com/megaeth-labs/revm/pull/1161))
- Add Hardhat to the "Used by" list ([#1164](https://github.com/megaeth-labs/revm/pull/1164))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/megaeth-labs/revm/pull/1140))
- Add VERBS to used by list ([#1141](https://github.com/megaeth-labs/revm/pull/1141))
- remove clone for context in handler_register.rs ([#1138](https://github.com/megaeth-labs/revm/pull/1138))
- Check runtime dynamically ([#1135](https://github.com/megaeth-labs/revm/pull/1135))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/megaeth-labs/revm/pull/1132))
- Add `db` and `db_mut` to evm ([#1133](https://github.com/megaeth-labs/revm/pull/1133))
- add ToString for no_std add exports some types in no_std ([#1128](https://github.com/megaeth-labs/revm/pull/1128))
- Add `clone` method to `ContextWithHandlerCfg` ([#1127](https://github.com/megaeth-labs/revm/pull/1127))
- remove unused EvmInstructionTables type alias ([#1123](https://github.com/megaeth-labs/revm/pull/1123))
- release tag v30 revm v6.1.0 ([#1100](https://github.com/megaeth-labs/revm/pull/1100))
- Ensure `L1Block` is in the cache ([#1121](https://github.com/megaeth-labs/revm/pull/1121))
- Fix feature name for generate_block_traces example ([#1120](https://github.com/megaeth-labs/revm/pull/1120))
- *(refactor)* Propagate fatal error ([#1116](https://github.com/megaeth-labs/revm/pull/1116))
- Revert "fix EthersDB deadlock ([#1089](https://github.com/megaeth-labs/revm/pull/1089))" ([#1118](https://github.com/megaeth-labs/revm/pull/1118))
- Remove DatabaseRef bound on CacheDB ([#1113](https://github.com/megaeth-labs/revm/pull/1113))
- clippy cleanup ([#1112](https://github.com/megaeth-labs/revm/pull/1112))
- *(deps)* bump anyhow from 1.0.79 to 1.0.80 ([#1108](https://github.com/megaeth-labs/revm/pull/1108))
- improve EIP-3155 implementation ([#1105](https://github.com/megaeth-labs/revm/pull/1105))
- release ([#1082](https://github.com/megaeth-labs/revm/pull/1082))
- *(state)* avoid cloning full account ([#1097](https://github.com/megaeth-labs/revm/pull/1097))
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/megaeth-labs/revm/pull/1085))
- Add memory offset ([#1032](https://github.com/megaeth-labs/revm/pull/1032))
- license date and revm docs ([#1080](https://github.com/megaeth-labs/revm/pull/1080))
- release ([#1067](https://github.com/megaeth-labs/revm/pull/1067))
- *(precompile)* make use of padding utilities, simplify secp256k1 ([#1073](https://github.com/megaeth-labs/revm/pull/1073))
- *(revm)* Add helpers to Build Revm with Context ([#1068](https://github.com/megaeth-labs/revm/pull/1068))
- *(revme)* statetests new format and return error ([#1066](https://github.com/megaeth-labs/revm/pull/1066))
- README path
- tag v27, revm v4.0.0 release ([#1061](https://github.com/megaeth-labs/revm/pull/1061))
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/megaeth-labs/revm/pull/1059))
- spelling on last commit ([#1058](https://github.com/megaeth-labs/revm/pull/1058))
- helper functions around Env ([#1057](https://github.com/megaeth-labs/revm/pull/1057))
- *(deps)* bump tokio from 1.35.1 to 1.36.0 ([#1052](https://github.com/megaeth-labs/revm/pull/1052))
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/megaeth-labs/revm/pull/1050))
- *(deps)* bump ethers-contract from 2.0.11 to 2.0.13 ([#1034](https://github.com/megaeth-labs/revm/pull/1034))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/megaeth-labs/revm/pull/1041))
- group handlers ([#1030](https://github.com/megaeth-labs/revm/pull/1030))
- *(Inspector)* add inspector depth test ([#1028](https://github.com/megaeth-labs/revm/pull/1028))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/megaeth-labs/revm/pull/1026))
- *(clippy)* nightly clippy ([#1025](https://github.com/megaeth-labs/revm/pull/1025))
- *(Execution)* Granular handles create/call,call_return,insert_call_outcome ([#1024](https://github.com/megaeth-labs/revm/pull/1024))
- *(Inspector)* Add return_memory_offset to Inspector::call ([#1006](https://github.com/megaeth-labs/revm/pull/1006))
- update call end docs ([#1000](https://github.com/megaeth-labs/revm/pull/1000))
- add getter for specId ([#998](https://github.com/megaeth-labs/revm/pull/998))
- Remove preserve_order in serde_json ([#997](https://github.com/megaeth-labs/revm/pull/997))
- update create docs ([#999](https://github.com/megaeth-labs/revm/pull/999))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/megaeth-labs/revm/pull/994))
- *(Inspector)* add CallOutcome to call/call_end ([#985](https://github.com/megaeth-labs/revm/pull/985))
- set deduct_caller in optimism handler ([#988](https://github.com/megaeth-labs/revm/pull/988))
- fix serde std flags for no-std build ([#987](https://github.com/megaeth-labs/revm/pull/987))
- *(Inspector)* Add CreateOutcome in create/create_end return ([#980](https://github.com/megaeth-labs/revm/pull/980))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/megaeth-labs/revm/pull/975))
- *(EvmBuilder)* Remove unnecessary BuilderStage trait ([#979](https://github.com/megaeth-labs/revm/pull/979))
- enhance readability ([#968](https://github.com/megaeth-labs/revm/pull/968))
- *(interpreter)* refactor sstore_cost ([#974](https://github.com/megaeth-labs/revm/pull/974))
- *(interpreter)* improve enum naming ([#962](https://github.com/megaeth-labs/revm/pull/962))
- *(deps)* bump anyhow from 1.0.77 to 1.0.79 ([#950](https://github.com/megaeth-labs/revm/pull/950))
- relax Bytes requirement and use slice instead ([#937](https://github.com/megaeth-labs/revm/pull/937))
- *(deps)* bump futures from 0.3.29 to 0.3.30 ([#927](https://github.com/megaeth-labs/revm/pull/927))
- *(deps)* bump anyhow from 1.0.75 to 1.0.76 ([#921](https://github.com/megaeth-labs/revm/pull/921))
- *(deps)* bump tokio from 1.34.0 to 1.35.0 ([#909](https://github.com/megaeth-labs/revm/pull/909))
- *(docs)* Update the benchmark docs to point to revm package ([#906](https://github.com/megaeth-labs/revm/pull/906))
- *(revm)* leverage StorageSlot methods, where appropriate ([#899](https://github.com/megaeth-labs/revm/pull/899))
- *(docs)* Update top-level benchmark docs ([#894](https://github.com/megaeth-labs/revm/pull/894))
- relax state generic ([#881](https://github.com/megaeth-labs/revm/pull/881))
- clippy ([#877](https://github.com/megaeth-labs/revm/pull/877))
- *(deps)* bump ethers-contract from 2.0.10 to 2.0.11 ([#867](https://github.com/megaeth-labs/revm/pull/867))
- bump k256 and use normalize_s ([#870](https://github.com/megaeth-labs/revm/pull/870))
- simplify use statements ([#864](https://github.com/megaeth-labs/revm/pull/864))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/megaeth-labs/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/megaeth-labs/revm/pull/859))
- *(deps)* bump tokio from 1.33.0 to 1.34.0 ([#856](https://github.com/megaeth-labs/revm/pull/856))
- change addresses to iterator and add into_addresses ([#855](https://github.com/megaeth-labs/revm/pull/855))
- use keccak256 for blockhash ([#854](https://github.com/megaeth-labs/revm/pull/854))
- review safety comments ([#811](https://github.com/megaeth-labs/revm/pull/811))
- *(deps)* bump futures from 0.3.28 to 0.3.29 ([#839](https://github.com/megaeth-labs/revm/pull/839))
- *(state)* consistent selfdestruct status transition ([#847](https://github.com/megaeth-labs/revm/pull/847))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/megaeth-labs/revm/pull/844))
- *(state)* simplify control flow in `CacheState::apply_evm_state` ([#842](https://github.com/megaeth-labs/revm/pull/842))
- Refactor precompile list from Hash to vec ([#823](https://github.com/megaeth-labs/revm/pull/823))
- *(state)* make `State::apply_transition` pub ([#832](https://github.com/megaeth-labs/revm/pull/832))
- *(state)* make bundle state non-optional ([#828](https://github.com/megaeth-labs/revm/pull/828))
- Refactor evm data to its file ([#815](https://github.com/megaeth-labs/revm/pull/815))
- for now support 1.69 rust compiler ([#814](https://github.com/megaeth-labs/revm/pull/814))
- refactor main return to handle ([#808](https://github.com/megaeth-labs/revm/pull/808))
- *(SharedMemory)* small refactor; tests ([#806](https://github.com/megaeth-labs/revm/pull/806))
- use `array::from_fn` in `make_instruction_table` ([#809](https://github.com/megaeth-labs/revm/pull/809))
- remove `step` and `step_end` return result ([#804](https://github.com/megaeth-labs/revm/pull/804))
- Instruction table ([#759](https://github.com/megaeth-labs/revm/pull/759))
- getter for  field of ([#792](https://github.com/megaeth-labs/revm/pull/792))
- Shared memory between calls ([#673](https://github.com/megaeth-labs/revm/pull/673))
- Fix typos ([#790](https://github.com/megaeth-labs/revm/pull/790))
- *(deps)* bump tokio from 1.32.0 to 1.33.0 ([#785](https://github.com/megaeth-labs/revm/pull/785))
- clang requirement ([#784](https://github.com/megaeth-labs/revm/pull/784))
- Use upstream create and create2 implementations ([#775](https://github.com/megaeth-labs/revm/pull/775))
- reorder JournalState impl ([#772](https://github.com/megaeth-labs/revm/pull/772))
- document everything, dedup existing docs ([#741](https://github.com/megaeth-labs/revm/pull/741))
- bump v26 revm v3.5.0 ([#765](https://github.com/megaeth-labs/revm/pull/765))
- document non-zero amounts in State::increment_balances ([#760](https://github.com/megaeth-labs/revm/pull/760))
- tag v25, revm v3.4.0 ([#755](https://github.com/megaeth-labs/revm/pull/755))
- Readme Updates ([#756](https://github.com/megaeth-labs/revm/pull/756))
- Improve wording and fix typos ([#749](https://github.com/megaeth-labs/revm/pull/749))
- say "warm" instead of "hot" ([#754](https://github.com/megaeth-labs/revm/pull/754))
- Logo ([#743](https://github.com/megaeth-labs/revm/pull/743))
- SELFDESTRUCT only in same transaction ([#719](https://github.com/megaeth-labs/revm/pull/719))
- error type for block header ([#731](https://github.com/megaeth-labs/revm/pull/731))
- refactor interpreter internals and cleanup ([#582](https://github.com/megaeth-labs/revm/pull/582))
- make BundleBuilder publicly available ([#729](https://github.com/megaeth-labs/revm/pull/729))
- *(perf)* only recalc code hash if its the default ([#716](https://github.com/megaeth-labs/revm/pull/716))
- add warning on panic conditions in take_bundle ([#715](https://github.com/megaeth-labs/revm/pull/715))
- Never inline the prepare functions ([#712](https://github.com/megaeth-labs/revm/pull/712))
- *(deps)* bump bytes from 1.4.0 to 1.5.0 ([#707](https://github.com/megaeth-labs/revm/pull/707))
- *(deps)* bump ethers-contract from 2.0.9 to 2.0.10 ([#705](https://github.com/megaeth-labs/revm/pull/705))
- *(state)* do not insert empty reverts in state ([#702](https://github.com/megaeth-labs/revm/pull/702))
- implement `Default` for other databases ([#691](https://github.com/megaeth-labs/revm/pull/691))
- make `impl Default for StateBuilder` generic ([#690](https://github.com/megaeth-labs/revm/pull/690))
- bundle size hint ([#670](https://github.com/megaeth-labs/revm/pull/670))
- deprecate `RefDBWrapper` ([#696](https://github.com/megaeth-labs/revm/pull/696))
- expose StateDBBox ([#694](https://github.com/megaeth-labs/revm/pull/694))
- *(state)* Make Database more generic. ([#687](https://github.com/megaeth-labs/revm/pull/687))
- nits and renamings ([#684](https://github.com/megaeth-labs/revm/pull/684))
- apply builder pattern for BundleState initialization ([#649](https://github.com/megaeth-labs/revm/pull/649))
- impl Eq, PartialEq for TransitionState ([#677](https://github.com/megaeth-labs/revm/pull/677))
- Removed the last dependencies breaking no-std build. ([#669](https://github.com/megaeth-labs/revm/pull/669))
- *(state)* bundle retention ([#666](https://github.com/megaeth-labs/revm/pull/666))
- *(deps)* bump ethers-contract from 2.0.8 to 2.0.9 ([#640](https://github.com/megaeth-labs/revm/pull/640))
- filter out empty bytecode from bundle ([#656](https://github.com/megaeth-labs/revm/pull/656))
- *(state)* bundle state split ([#646](https://github.com/megaeth-labs/revm/pull/646))
- add value parameter to Inspector::selfdestruct ([#645](https://github.com/megaeth-labs/revm/pull/645))
- Split transaction pre verification to separate function ([#573](https://github.com/megaeth-labs/revm/pull/573))
- *(state)* bundle selfdestructs ([#627](https://github.com/megaeth-labs/revm/pull/627))
- misc improvements ([#633](https://github.com/megaeth-labs/revm/pull/633))
- bundle state ([#637](https://github.com/megaeth-labs/revm/pull/637))
- *(state)* remove redundunt info revert on destruct ([#635](https://github.com/megaeth-labs/revm/pull/635))
- book workflow ([#537](https://github.com/megaeth-labs/revm/pull/537))
- *(state)* bundle reverts collapse ([#626](https://github.com/megaeth-labs/revm/pull/626))
- Revert "feat: alloy migration ([#535](https://github.com/megaeth-labs/revm/pull/535))" ([#616](https://github.com/megaeth-labs/revm/pull/616))
- *(state)* account & storage revert value preservation ([#614](https://github.com/megaeth-labs/revm/pull/614))
- spell check ([#615](https://github.com/megaeth-labs/revm/pull/615))
- get or insert bundle state ([#613](https://github.com/megaeth-labs/revm/pull/613))
- *(deps)* bump anyhow from 1.0.74 to 1.0.75 ([#606](https://github.com/megaeth-labs/revm/pull/606))
- *(deps)* bump tokio from 1.31.0 to 1.32.0 ([#607](https://github.com/megaeth-labs/revm/pull/607))
- export some `unreachable_pub` items ([#598](https://github.com/megaeth-labs/revm/pull/598))
- *(deps)* bump anyhow from 1.0.72 to 1.0.74 ([#602](https://github.com/megaeth-labs/revm/pull/602))
- fix test build, use new types ([#605](https://github.com/megaeth-labs/revm/pull/605))
- Revert test, not change storage check , renaming of original slot value ([#601](https://github.com/megaeth-labs/revm/pull/601))
- `TransitionState::with_capacity` -> `TransitionState::single` ([#600](https://github.com/megaeth-labs/revm/pull/600))
- pre-allocate inner bundle state ([#599](https://github.com/megaeth-labs/revm/pull/599))
- avoid unnecessary allocations ([#581](https://github.com/megaeth-labs/revm/pull/581))
- *(deps)* bump tokio from 1.29.1 to 1.31.0 ([#595](https://github.com/megaeth-labs/revm/pull/595))
- move precompiles to EVMData for inspectors ([#588](https://github.com/megaeth-labs/revm/pull/588))
- rewrite revm-test as a criterion bench ([#579](https://github.com/megaeth-labs/revm/pull/579))
- clippy and fmt ([#568](https://github.com/megaeth-labs/revm/pull/568))
- optimize stack usage for recursive `call` and `create` programs ([#522](https://github.com/megaeth-labs/revm/pull/522))
- consume all gas on invalid opcode ([#500](https://github.com/megaeth-labs/revm/pull/500))
- *(deps)* bump auto_impl from 1.0.1 to 1.1.0 ([#478](https://github.com/megaeth-labs/revm/pull/478))
- fix comment typo ([#517](https://github.com/megaeth-labs/revm/pull/517))
- add some CacheDB docs ([#484](https://github.com/megaeth-labs/revm/pull/484))
- refactor interpreter run and remove static flag ([#481](https://github.com/megaeth-labs/revm/pull/481))
- Bundle inspector crate/call calls ([#480](https://github.com/megaeth-labs/revm/pull/480))
- Bump v24, revm v3.3.0 ([#476](https://github.com/megaeth-labs/revm/pull/476))
- *(deps)* bump tokio from 1.27.0 to 1.28.0 ([#470](https://github.com/megaeth-labs/revm/pull/470))
- *(deps)* bump anyhow from 1.0.70 to 1.0.71 ([#471](https://github.com/megaeth-labs/revm/pull/471))
- add example to revm crate ([#468](https://github.com/megaeth-labs/revm/pull/468))
- Release v23, revm v3.2.0 ([#464](https://github.com/megaeth-labs/revm/pull/464))
- Release v22, revm v3.1.1 ([#460](https://github.com/megaeth-labs/revm/pull/460))
- typos ([#448](https://github.com/megaeth-labs/revm/pull/448))
- v21, revm v3.1.0 ([#444](https://github.com/megaeth-labs/revm/pull/444))
- bump all
- ethers to v2.0
- Improve EthersDB::new ([#440](https://github.com/megaeth-labs/revm/pull/440))
- add feature for ignoring base fee check ([#436](https://github.com/megaeth-labs/revm/pull/436))
- Update README.md ([#424](https://github.com/megaeth-labs/revm/pull/424))
- *(deps)* bump futures from 0.3.26 to 0.3.27 ([#416](https://github.com/megaeth-labs/revm/pull/416))
- enabled primtive default feature in precompile ([#409](https://github.com/megaeth-labs/revm/pull/409))
- fix typo StorageChange ([#403](https://github.com/megaeth-labs/revm/pull/403))
- *(deps)* bump tokio from 1.25.0 to 1.26.0 ([#395](https://github.com/megaeth-labs/revm/pull/395))
- remove gas blocks ([#391](https://github.com/megaeth-labs/revm/pull/391))
- impl NonceTooHigh/ NonceTooLow checks ([#383](https://github.com/megaeth-labs/revm/pull/383))
- fix compilation if serde not enabled ([#381](https://github.com/megaeth-labs/revm/pull/381))
- add no_std to primitives ([#366](https://github.com/megaeth-labs/revm/pull/366))
- *(deps)* bump tokio from 1.24.2 to 1.25.0 ([#352](https://github.com/megaeth-labs/revm/pull/352))
- *(deps)* bump futures from 0.3.25 to 0.3.26 ([#353](https://github.com/megaeth-labs/revm/pull/353))
- rename Then to Than ([#368](https://github.com/megaeth-labs/revm/pull/368))
- add error details to InvalidTransaction::LackOfFundForGasLimit ([#364](https://github.com/megaeth-labs/revm/pull/364))
- Use gas price in place of effective gas price for initial balance check ([#359](https://github.com/megaeth-labs/revm/pull/359))
- revm-precompiles to revm-precompile
- Bump v20, changelog ([#350](https://github.com/megaeth-labs/revm/pull/350))
- Cleanup imports ([#348](https://github.com/megaeth-labs/revm/pull/348))
- add logs & return value to revert ([#343](https://github.com/megaeth-labs/revm/pull/343))
- includes to libs ([#338](https://github.com/megaeth-labs/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/megaeth-labs/revm/pull/334))
- mark `with-serde` feature as deprecated ([#328](https://github.com/megaeth-labs/revm/pull/328))
- add Eq to AccountState ([#326](https://github.com/megaeth-labs/revm/pull/326))
- make load_account pub ([#325](https://github.com/megaeth-labs/revm/pull/325))
- Cleanup, move hot fields toggether in Interpreter ([#321](https://github.com/megaeth-labs/revm/pull/321))
- *(deps)* bump tokio from 1.22.0 to 1.23.0 ([#284](https://github.com/megaeth-labs/revm/pull/284))
- remove --all-features from tests bcs of gas calc gets disabled ([#291](https://github.com/megaeth-labs/revm/pull/291))
- Integer overflow while calculating the remaining gas in GasInspector ([#287](https://github.com/megaeth-labs/revm/pull/287))
- native bits ([#278](https://github.com/megaeth-labs/revm/pull/278))
- *(release)* Bump revm and precompiles versions
- Bump primitive_types. Add statetest spec
- Bump revm to v2.3.0
- disable gas refunds with env flag ([#267](https://github.com/megaeth-labs/revm/pull/267))
- Export StorageSlot ([#265](https://github.com/megaeth-labs/revm/pull/265))
- Remove unused parking_lot dependency ([#244](https://github.com/megaeth-labs/revm/pull/244))
- v17 release notes, revm v2.2.0 ([#262](https://github.com/megaeth-labs/revm/pull/262))
- *(eth/test)* Added OEF spec for tests. Skip HighGasPrice ([#261](https://github.com/megaeth-labs/revm/pull/261))
- if returndatacopy is len 0 return after initial cost ([#259](https://github.com/megaeth-labs/revm/pull/259))
- typos ([#232](https://github.com/megaeth-labs/revm/pull/232))
- Borrow self and add derive traits for OpCode ([#231](https://github.com/megaeth-labs/revm/pull/231))
- Bump revm v2.1.0 ([#224](https://github.com/megaeth-labs/revm/pull/224))
- expose original value on storageslot ([#216](https://github.com/megaeth-labs/revm/pull/216))
- revm bump v2.0.0, precompile bump v1.1.1 ([#212](https://github.com/megaeth-labs/revm/pull/212))
- Optimize gas calculation U256 to u64 ([#213](https://github.com/megaeth-labs/revm/pull/213))
- current_opcode fn and rename program_counter to instruction_pointer ([#211](https://github.com/megaeth-labs/revm/pull/211))
- Cfg choose create analysis, option on bytecode size limit ([#210](https://github.com/megaeth-labs/revm/pull/210))
- export create address calls ([#209](https://github.com/megaeth-labs/revm/pull/209))
- Cargo sort. Bump lib versions ([#208](https://github.com/megaeth-labs/revm/pull/208))
- *(deps)* bump futures from 0.3.23 to 0.3.24 ([#194](https://github.com/megaeth-labs/revm/pull/194))
- use Infallible for memory db's error type ([#196](https://github.com/megaeth-labs/revm/pull/196))
- reexport revm_precompiles as precompiles ([#197](https://github.com/megaeth-labs/revm/pull/197))
- Cache precompile hashmaps ([#192](https://github.com/megaeth-labs/revm/pull/192))
- Add support for old forks. ([#191](https://github.com/megaeth-labs/revm/pull/191))
- export JournaledState ([#190](https://github.com/megaeth-labs/revm/pull/190))
- Revert "refactor(revm): use u64 for gas refund counter ([#180](https://github.com/megaeth-labs/revm/pull/180))" ([#187](https://github.com/megaeth-labs/revm/pull/187))
- *(revm)* use u64 for gas refund counter ([#180](https://github.com/megaeth-labs/revm/pull/180))
- *(precompiles)* Vec -> BTreeMap ([#177](https://github.com/megaeth-labs/revm/pull/177))
- *(deps)* bump futures from 0.3.21 to 0.3.23 ([#173](https://github.com/megaeth-labs/revm/pull/173))
- Handle HighNonce tests ([#176](https://github.com/megaeth-labs/revm/pull/176))
- JournaledState ([#175](https://github.com/megaeth-labs/revm/pull/175))
- Update account storage methods in CacheDB ([#171](https://github.com/megaeth-labs/revm/pull/171))
- Return `ExecutionResult`, which includes `gas_refunded` ([#169](https://github.com/megaeth-labs/revm/pull/169))
- Optimize calldataload. Some cleanup ([#168](https://github.com/megaeth-labs/revm/pull/168))
- bump revm v1.9.0
- Bytecode hash, remove override_spec, ([#165](https://github.com/megaeth-labs/revm/pull/165))
- revm bump 1.8. update libs. snailtracer rename ([#159](https://github.com/megaeth-labs/revm/pull/159))
- bump revm_precompiles to v1.1.0
- bump revm to v1.7.0
- Byzantine to Byzantium ([#146](https://github.com/megaeth-labs/revm/pull/146))
- Make CacheDB fields pub ([#145](https://github.com/megaeth-labs/revm/pull/145))
- Update README
- bump revm 1.6.0, changelogs
- Introduce account Touched/Cleared/None state in CacheDB ([#140](https://github.com/megaeth-labs/revm/pull/140))
- Return specific `Return` statuses in `CALL`s ([#136](https://github.com/megaeth-labs/revm/pull/136))
- don't delete account and storage entries on commit ([#126](https://github.com/megaeth-labs/revm/pull/126))
- revm bump v1.5.0. Release notes
- enable EIP2200 in Istanbul ([#125](https://github.com/megaeth-labs/revm/pull/125))
- Consensus error with gas block for SSTORE stipend check ([#124](https://github.com/megaeth-labs/revm/pull/124))
- v9 release notes
- export evm_inner ([#122](https://github.com/megaeth-labs/revm/pull/122))
- *(deps)* bump auto_impl from 0.5.0 to 1.0.1 ([#118](https://github.com/megaeth-labs/revm/pull/118))
- [revm] bump to v1.4.0
- *(clippy)* make clippy happy ([#120](https://github.com/megaeth-labs/revm/pull/120))
- rm empty line
- changelog for v7 tag
- empty keccak constant and remove access_list.clone ([#111](https://github.com/megaeth-labs/revm/pull/111))
- typo fixes
- [revm] is_static for Inspector initialize_interp
- v6 changelog, bump versions
- add missing derives
- fix readme typo
- Rework analysis ([#89](https://github.com/megaeth-labs/revm/pull/89))
- Update AccountInfo#code documentation ([#94](https://github.com/megaeth-labs/revm/pull/94))
- Various fixes ([#93](https://github.com/megaeth-labs/revm/pull/93))
- remove untable fmt
- nit
- [revm] optimize is_push check
- turn off default features for zkp-u256 ([#68](https://github.com/megaeth-labs/revm/pull/68))
- Bump futures from 0.3.17 to 0.3.21 ([#64](https://github.com/megaeth-labs/revm/pull/64))
- bump dependencies ([#63](https://github.com/megaeth-labs/revm/pull/63))
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/megaeth-labs/revm/pull/52))
- provide default impl for inspector trait
- improve docs slightly
- document unsafe code in stack
- [revm] pop_top and unsafe comments ([#51](https://github.com/megaeth-labs/revm/pull/51))
- clippy
- Inspector fixup
- Bump precompiles to v0.4.0 bump revm v1.2.0
- [recompl] Bump precompile deps, cargo sort on workspace
- fix lints
- cargo fmt
- [revm_precompiles] added flag for k256 lib
- [revm] Bump to v1.1.0
- Bug fix for unknown OpCode
- internal cleanups
- remove unneeded comments from DB
- [revm] output log. Stetetest test log output. fmt
- Bump versions, Changelogs, fmt, revm readme, clippy.
- bugfix jumpdest
- optimize, remove inlines
- GasBlock for all Spec
- [revm] Run test multiple times. fmt, BenchmarkDB
- [revm] wip multiple u256
- [revm] merge parity u256 with zkpu256
- [revm][perf] GasBlock analazis and optimizations.
- handle empty contract
- wip
- [revm] precalculated gas blocks
- [revme][bugfix] fix PC opcode from previous commit
- [revm] Optimize PC, some perf
- [revme][debugger] stack pop/push
- [revme][debug] some print cli
- readme. debuger update
- [revm] Rename Handler to Host
- [revm] Simplified host inspector
- [revme] initial commit. Cmd skeleton added.statetests moved
- Multiple changes: web3 db, debugger initial commit, precompile load
- [revm][insp] now can derail inner_call
- revm v0.5 readme
- Bump revm v0.5, precompiles v0.3. some Cleanup
- [revm][perf] push slice optimized
- [revm][perf] memory set optimized
- [revm][perf] mload as slice
- Optimize memory calculation
- Memory to usize, clippy,fmt
- wip optimize i256
- TEMP switch stacks H256 with U256
- replace xx::zero() with is_zero()
- [revm][perf] Error refactor to Return ([#9](https://github.com/megaeth-labs/revm/pull/9))
- [revm][perf] remove u8 casting
- god clippy accepts our sacrifice.
- [revm] some perfs
- [revm] Perfs stack pop. Benchmark snailtracer.
- fmt
- [revm] Bump auto_impl to v0.5
- [revm] cleanup
- [revm] Rename opcode to instruction. Opcode is u8 now
- [revm] NO_GAS_MEASURING feature. inline always opcodes
- [revm] USE_GAS added in Spec
- Remove ethereumjs-util js file
- Bump v1. cleanup, changelogs.
- [revmjs] evm functions added. still need to debug example
- LICENSE. wasm example
- fmt
- [wasm] simple example
- [revm] fix build. Hashbrown features
- wip wasm. Remove parity-crypto for secp256k1 and made it as feature.
- wip wasm
- Database traits made useful.
- EVM Interface changed. Inspector called separately
- [revm] tweak
- Bump revm v0.3.1
- unused imports
- fmt
- Bump revm v0.3.0. README updated
- DB ref mut polished
- And now we debug
- [revm] Interface. Inspector added, Env cleanup. revm-test passes
- BIG interface change
- no_sdt to no_std
- [precompiles] spelling, small cleanup
- README, CHANGELOG added for revm crate
- Bump revm to v0.2.1
- Precompiles readme. fix for workspace
- BIG reorg. workspace added. revm-precompile lib
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

## [40.0.3](https://github.com/bluealloy/revm/compare/revm-v40.0.2...revm-v40.0.3) - 2026-05-26

### Other

- updated the following local packages: revm-database, revm-context, revm-handler, revm-inspector, revm-statetest-types

## [40.0.2](https://github.com/bluealloy/revm/compare/revm-v40.0.1...revm-v40.0.2) - 2026-05-22

### Other

- updated the following local packages: revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector, revm-statetest-types

## [40.0.1](https://github.com/bluealloy/revm/compare/revm-v40.0.0...revm-v40.0.1) - 2026-05-21

### Other

- updated the following local packages: revm-context-interface, revm-interpreter, revm-precompile, revm-handler, revm-context, revm-inspector, revm-statetest-types

## [40.0.0](https://github.com/bluealloy/revm/compare/revm-v38.0.0...revm-v40.0.0) - 2026-05-19

### Other

- backport v107 release notes from branch ([#3617](https://github.com/bluealloy/revm/pull/3617))
- rm op-revm (migrated to ethereum-optimism/optimism) ([#3568](https://github.com/bluealloy/revm/pull/3568))

## [38.0.0](https://github.com/bluealloy/revm/compare/revm-v37.0.0...revm-v38.0.0) - 2026-04-17

### Other

- updated the following local packages: revm-state, revm-context-interface, revm-interpreter, revm-precompile, revm-handler, revm-inspector, revm-database-interface, revm-context, revm-database, revm-statetest-types

## [37.0.0](https://github.com/bluealloy/revm/compare/revm-v36.0.0...revm-v37.0.0) - 2026-04-10

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector, revm-statetest-types, revm-state

## [36.0.0](https://github.com/bluealloy/revm/compare/revm-v35.0.0...revm-v36.0.0) - 2026-03-04

### Other

- bump revm-database-interface to v10.0.0

## [35.0.0](https://github.com/bluealloy/revm/compare/revm-v34.0.0...revm-v35.0.0) - 2026-03-02

### Added

- *(precompile)* add aws-lc-rs as alternative backend for secp256r1 ([#3451](https://github.com/bluealloy/revm/pull/3451))

### Other

- remove GPL mention and update gmp feature comments ([#3383](https://github.com/bluealloy/revm/pull/3383))

## [34.0.0](https://github.com/bluealloy/revm/compare/revm-v33.1.0...revm-v34.0.0) - 2026-01-15

### Added

- Propagate `map-foldhash` Feature Through Dependency Chain ([#3252](https://github.com/bluealloy/revm/pull/3252))
- dynamically link gmp ([#3250](https://github.com/bluealloy/revm/pull/3250))

### Other

- fix typos, grammar errors, and improve documentation consistency ([#3294](https://github.com/bluealloy/revm/pull/3294))
- happy new year, 2026 licence ([#3272](https://github.com/bluealloy/revm/pull/3272))
- re-export statetest-types from revm crate behind test-types feature ([#3247](https://github.com/bluealloy/revm/pull/3247))
- *(fmt)* merge all imports ([#3184](https://github.com/bluealloy/revm/pull/3184))

## [33.1.0](https://github.com/bluealloy/revm/compare/revm-v33.0.0...revm-v33.1.0) - 2025-11-14

### Fixed

- expose optional_priority_fee_check feature from context crate ([#3159](https://github.com/bluealloy/revm/pull/3159))

## [33.0.0](https://github.com/bluealloy/revm/compare/revm-v31.0.2...revm-v33.0.0) - 2025-11-10

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [31.0.2](https://github.com/bluealloy/revm/compare/revm-v31.0.1...revm-v31.0.2) - 2025-11-10

### Other

- updated the following local packages: revm-database, revm-context, revm-handler, revm-inspector

## [31.0.1](https://github.com/bluealloy/revm/compare/revm-v31.0.0...revm-v31.0.1) - 2025-11-07

### Other

- updated the following local packages: revm-primitives, revm-context, revm-bytecode, revm-state, revm-database-interface, revm-context-interface, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [31.0.0](https://github.com/bluealloy/revm/compare/revm-v30.2.0...revm-v31.0.0) - 2025-10-30

### Other

- consolidate revme imports ([#3088](https://github.com/bluealloy/revm/pull/3088))

## [30.2.0](https://github.com/bluealloy/revm/compare/revm-v30.1.2...revm-v30.2.0) - 2025-10-17

### Other

- updated the following local packages: revm-interpreter, revm-handler, revm-inspector

## [30.1.2](https://github.com/bluealloy/revm/compare/revm-v30.1.1...revm-v30.1.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-handler, revm-inspector

## [30.1.1](https://github.com/bluealloy/revm/compare/revm-v30.1.0...revm-v30.1.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context, revm-interpreter, revm-precompile, revm-handler, revm-state, revm-database-interface, revm-context-interface, revm-database, revm-inspector

## [30.1.0](https://github.com/bluealloy/revm/compare/revm-v30.0.0...revm-v30.1.0) - 2025-10-09

### Other

- updated the following local packages: revm-database-interface, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector, revm-context-interface, revm-context

## [30.0.0](https://github.com/bluealloy/revm/compare/revm-v29.0.1...revm-v30.0.0) - 2025-10-07

### Added

- allow EIP-7623 to be disabled ([#2985](https://github.com/bluealloy/revm/pull/2985))
- [**breaking**] Remove kzg-rs ([#2909](https://github.com/bluealloy/revm/pull/2909))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- forward optional_fee_charge feature ([#3005](https://github.com/bluealloy/revm/pull/3005))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))

## [29.0.1](https://github.com/bluealloy/revm/compare/revm-v29.0.0...revm-v29.0.1) - 2025-09-23

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter, revm-handler, revm-inspector

## [29.0.0](https://github.com/bluealloy/revm/compare/revm-v28.0.1...revm-v29.0.0) - 2025-08-23

### Other

- updated the following local packages: revm-bytecode, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-state, revm-inspector

## [28.0.1](https://github.com/bluealloy/revm/compare/revm-v28.0.0...revm-v28.0.1) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-context-interface, revm-database, revm-precompile, revm-inspector, revm-database-interface, revm-context, revm-interpreter, revm-handler

## [28.0.0](https://github.com/bluealloy/revm/compare/revm-v27.1.0...revm-v28.0.0) - 2025-08-06

### Added

- Reuse bls12-381 codepaths to implement kzg point evaluation precompile ([#2809](https://github.com/bluealloy/revm/pull/2809))
- refactor test utils ([#2813](https://github.com/bluealloy/revm/pull/2813))
- add system transaction inspection support ([#2808](https://github.com/bluealloy/revm/pull/2808))

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- reuse global crypto provide idea ([#2786](https://github.com/bluealloy/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [27.1.0](https://github.com/bluealloy/revm/compare/revm-v27.0.3...revm-v27.1.0) - 2025-07-23

### Added

- expose sha3-keccak in revm and revm-primitives ([#2713](https://github.com/bluealloy/revm/pull/2713))

### Fixed

- features and check in ci ([#2766](https://github.com/bluealloy/revm/pull/2766))
- gas deduction with `disable_balance_check` ([#2699](https://github.com/bluealloy/revm/pull/2699))

### Other

- add asm-sha2 feature for sha2 precompile ([#2712](https://github.com/bluealloy/revm/pull/2712))

## [27.0.3](https://github.com/bluealloy/revm/compare/revm-v27.0.2...revm-v27.0.3) - 2025-07-14

### Other

- updated the following local packages: revm-context, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [27.0.2](https://github.com/bluealloy/revm/compare/revm-v27.0.1...revm-v27.0.2) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode, revm-handler, revm-inspector, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter

## [27.0.1](https://github.com/bluealloy/revm/compare/revm-v26.0.1...revm-v27.0.1) - 2025-06-30

### Added

- optional_eip3541 ([#2661](https://github.com/bluealloy/revm/pull/2661))

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))
- inline documentation of revm top modules ([#2666](https://github.com/bluealloy/revm/pull/2666))
- use TxEnv::builder ([#2652](https://github.com/bluealloy/revm/pull/2652))

## [26.0.1](https://github.com/bluealloy/revm/compare/revm-v26.0.0...revm-v26.0.1) - 2025-06-20

### Fixed

- call stack_frame.clear() at end ([#2656](https://github.com/bluealloy/revm/pull/2656))

## [26.0.0](https://github.com/bluealloy/revm/compare/revm-v25.0.0...revm-v26.0.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- enable P256 in Osaka ([#2601](https://github.com/bluealloy/revm/pull/2601))

### Fixed

- compare_or_save_testdata sig ([#2637](https://github.com/bluealloy/revm/pull/2637))

### Other

- lints for examples ([#2650](https://github.com/bluealloy/revm/pull/2650))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [25.0.0](https://github.com/bluealloy/revm/compare/revm-v24.0.1...revm-v25.0.0) - 2025-06-06

### Added

- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Fixed

- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/bluealloy/revm/pull/2581))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))

## [24.0.1](https://github.com/bluealloy/revm/compare/revm-v24.0.0...revm-v24.0.1) - 2025-05-31

### Other

- updated the following local packages: revm-context, revm-handler, revm-inspector

## [24.0.0](https://github.com/bluealloy/revm/compare/revm-v23.1.0...revm-v24.0.0) - 2025-05-22

### Other

- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))

## [23.1.0](https://github.com/bluealloy/revm/compare/revm-v23.0.0...revm-v23.1.0) - 2025-05-07

Dependency bump

## [23.0.0](https://github.com/bluealloy/revm/compare/revm-v22.0.1...revm-v23.0.0) - 2025-05-07

### Other

- typos ([#2474](https://github.com/bluealloy/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [22.0.1](https://github.com/bluealloy/revm/compare/revm-v22.0.0..revm-v22.0.1) - 2025-04-15

### Other

## [22.0.0](https://github.com/bluealloy/revm/compare/revm-v21.0.0...revm-v22.0.0) - 2025-04-09

### Added

- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))
- Reorder revm deps in Cargo.toml ([#2331](https://github.com/bluealloy/revm/pull/2331))

## [21.0.0](https://github.com/bluealloy/revm/compare/revm-v20.0.0...revm-v21.0.0) - 2025-03-28

### Added

- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))
- Add a wrapper for arkworks for EIP196 ([#2305](https://github.com/bluealloy/revm/pull/2305))

### Other

- Propagate asyncdb feature flag from database-interface to revm  ([#2310](https://github.com/bluealloy/revm/pull/2310))

## [20.0.0](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.7...revm-v20.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database, revm-precompile

## [20.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.6...revm-v20.0.0-alpha.7) - 2025-03-21

### Other

- updated the following local packages: revm-primitives, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [20.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.5...revm-v20.0.0-alpha.6) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context-interface, revm-context, revm-precompile, revm-handler, revm-inspector

## [20.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.4...revm-v20.0.0-alpha.5) - 2025-03-12

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter, revm-handler, revm-inspector

## [20.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.3...revm-v20.0.0-alpha.4) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [20.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.2...revm-v20.0.0-alpha.3) - 2025-03-10

### Fixed

- *(precompiles)* add portable flag for bls ([#2174](https://github.com/bluealloy/revm/pull/2174))

## [20.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.1...revm-v20.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- examples to use main revm crate ([#2152](https://github.com/bluealloy/revm/pull/2152))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/bluealloy/revm/pull/2132))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [20.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revm-v19.5.0...revm-v20.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- integrate alloy-eips (#2078)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- integrate codspeed (#1935)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- push NonceChange to Journal in deduct_caller (#1804)
- Merge validation/analyzis with Bytecode (#1793)
- Restructuring Part3 inspector crate (#1788)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- to_plain_state (#1778)
- *(example)* deploy bytecode from scratch (#1767)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Fixed

- make macro crate-agnostic (#1802)

### Other

- backport op l1 fetch perf (#2076)
- API cleanup (#2067)
- Add helpers with_inspector with_precompile (#2063)
- Bump licence year to 2025 (#2058)
- bump alloy versions to match latest (#2007)
- align crates versions (#1983)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- EVM transact, make output generic for POSTEXEC (#1931)
- Move CfgEnv from context-interface to context crate (#1910)
- bumps select alloy crates to 0.6 (#1854)
- some no_std cleanup (#1834)
- bump alloy to 0.4.2 (#1817)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
- *(deps)* bump anyhow from 1.0.88 to 1.0.89 (#1772)
- simplify SuccessOrHalt trait bound (#1768)


## [19.5.0](https://github.com/bluealloy/revm/compare/revm-v19.4.0...revm-v19.5.0) - 2025-02-11

### Fixed

- *(op-isthmus)* missing granite precompiles and disable eof (#2065)
- *(op)* add operator fee scalar scaling factor on refund (#2050)
- fix ethersdb.rs forking ([#2047](https://github.com/bluealloy/revm/pull/2047))

## [19.4.0](https://github.com/bluealloy/revm/compare/revm-v19.3.0...revm-v19.4.0) - 2025-01-28

### Added

- Include EIP7702 address for extcodecopy ([#2016](https://github.com/bluealloy/revm/pull/2016))
- add precompiles for BLS12-381 to isthmus ([#2000](https://github.com/bluealloy/revm/pull/2000))

### Fixed

- *(op)* isthmus compilation ([#2022](https://github.com/bluealloy/revm/pull/2022))

## [19.3.0](https://github.com/bluealloy/revm/compare/revm-v19.2.0...revm-v19.3.0) - 2025-01-13

### Added

- *(EIP-7623)* adjuct floor gas check order ([#1990](https://github.com/bluealloy/revm/pull/1990))

## [19.2.0](https://github.com/bluealloy/revm/compare/revm-v19.1.0...revm-v19.2.0) - 2025-01-06

### Fixed

- *(op)* move l1block loading to verification ([#1970](https://github.com/bluealloy/revm/pull/1970))

## [19.1.0](https://github.com/bluealloy/revm/compare/revm-v19.0.0...revm-v19.1.0) - 2025-01-06

### Added

- handle isthmus operator fee ([#1960](https://github.com/bluealloy/revm/pull/1960))

### Other

- optimize tx l1 fetches ([#1967](https://github.com/bluealloy/revm/pull/1967))

## [19.0.0](https://github.com/bluealloy/revm/compare/revm-v18.0.0...revm-v19.0.0) - 2024-12-26

### Added

- apply latest EIP-7702 changes ([#1850](https://github.com/bluealloy/revm/pull/1850))
- *(Prague)* EIP-7623 Increase Calldata Cost ([#1744](https://github.com/bluealloy/revm/pull/1744))

### Other

- eip7702 chainid u256 change ([#1950](https://github.com/bluealloy/revm/pull/1950))
- refactor L1BlockInfo::tx_estimated_size_fjord ([#1862](https://github.com/bluealloy/revm/pull/1862))
- [v50] perf: remove unneeded clones ([#1863](https://github.com/bluealloy/revm/pull/1863))
- Add static Optimism precompiles for Fjord & Granite ([#1864](https://github.com/bluealloy/revm/pull/1864))

## [18.0.0](https://github.com/bluealloy/revm/compare/revm-v17.1.0...revm-v18.0.0) - 2024-11-06

### Added

- to_plain_state ([#1778](https://github.com/bluealloy/revm/pull/1778)) ([#1841](https://github.com/bluealloy/revm/pull/1841))

### Other

- bump alloy-eip7702 and remove `Parity` re-export ([#1842](https://github.com/bluealloy/revm/pull/1842))
- Osaka Activation (release_49 branch) ([#1835](https://github.com/bluealloy/revm/pull/1835))

## [17.1.0](https://github.com/bluealloy/revm/compare/revm-v17.0.0...revm-v17.1.0) - 2024-10-23

### Other

- chore: bump alloydb deps ([#1832](https://github.com/bluealloy/revm/pull/1832))

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-v16.0.0...revm-v17.0.0) - 2024-10-23

### Other

- bump alloy-eip7702 ([#1829](https://github.com/bluealloy/revm/pull/1829))

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-v15.0.0...revm-v16.0.0) - 2024-10-17

### Other

- update Cargo.toml dependencies

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-v14.0.3...revm-v15.0.0) - 2024-10-17

### Added

- Rename PRAGUE_EOF to OSAKA ([#1822](https://github.com/bluealloy/revm/pull/1822))
- *(EIP-7702)* devnet-4 changes ([#1821](https://github.com/bluealloy/revm/pull/1821))

## [14.0.3](https://github.com/bluealloy/revm/compare/revm-v14.0.2...revm-v14.0.3) - 2024-09-26

### Other

- update Cargo.lock dependencies

## [14.0.2](https://github.com/bluealloy/revm/compare/revm-v14.0.1...revm-v14.0.2) - 2024-09-18

### Fixed

- *(inspector)* always call selfdestruct if entry is made ([#1746](https://github.com/bluealloy/revm/pull/1746))

### Other

- *(deps)* bump alloy-sol-types from 0.8.0 to 0.8.2 ([#1762](https://github.com/bluealloy/revm/pull/1762))
- *(deps)* bump anyhow from 1.0.86 to 1.0.87 ([#1760](https://github.com/bluealloy/revm/pull/1760))
- make clippy happy ([#1755](https://github.com/bluealloy/revm/pull/1755))
- Test l1 gas used fjord ([#1749](https://github.com/bluealloy/revm/pull/1749))
- Add test for `revm::optimism::L1BlockInfo::calculate_tx_l1_cost_fjord` ([#1743](https://github.com/bluealloy/revm/pull/1743))
- *(deps)* bump tokio from 1.39.2 to 1.40.0 ([#1739](https://github.com/bluealloy/revm/pull/1739))

## [14.0.1](https://github.com/bluealloy/revm/compare/revm-v14.0.0...revm-v14.0.1) - 2024-08-30

### Other
- Bump new logo ([#1735](https://github.com/bluealloy/revm/pull/1735))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-v13.0.0...revm-v14.0.0) - 2024-08-29

### Added
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/bluealloy/revm/pull/1695))

### Other
- *(deps)* bump alloy and primitives ([#1725](https://github.com/bluealloy/revm/pull/1725))
- cast block number to u64 and not usize ([#1727](https://github.com/bluealloy/revm/pull/1727))
- clean up some journalstate docs ([#1712](https://github.com/bluealloy/revm/pull/1712))
- update some docs related to state ([#1711](https://github.com/bluealloy/revm/pull/1711))

## [12.2.0](https://github.com/bluealloy/revm/compare/revm-v12.1.0...revm-v12.2.0) - 2024-08-08

### Added
- check for typos in CI ([#1686](https://github.com/bluealloy/revm/pull/1686))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/bluealloy/revm/pull/1648))

### Other
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/bluealloy/revm/pull/1685))
- *(deps)* bump rstest from 0.21.0 to 0.22.0 ([#1681](https://github.com/bluealloy/revm/pull/1681))
- *(deps)* bump tokio from 1.38.1 to 1.39.2 ([#1668](https://github.com/bluealloy/revm/pull/1668))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/bluealloy/revm/pull/1661))
- avoid cloning original_bytes ([#1646](https://github.com/bluealloy/revm/pull/1646))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/bluealloy/revm/pull/1621))
- *(eof)* simplify magic checks ([#1633](https://github.com/bluealloy/revm/pull/1633))

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-v11.0.0...revm-v12.0.0) - 2024-07-16

### Added
- pass interpreter into Inspector::log ([#1610](https://github.com/bluealloy/revm/pull/1610))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bluealloy/revm/pull/1607))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/bluealloy/revm/pull/1558))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))
- correctly calculate eofcreate address ([#1619](https://github.com/bluealloy/revm/pull/1619))
- allow non-static lifetime in HandleRegisterBox ([#1608](https://github.com/bluealloy/revm/pull/1608))
- *(EOF)* Use cfg code size limit for eofcreate ([#1606](https://github.com/bluealloy/revm/pull/1606))

### Other
- bump alloy deps ([#1623](https://github.com/bluealloy/revm/pull/1623))
- *(deps)* bump alloy-sol-types from 0.7.6 to 0.7.7 ([#1614](https://github.com/bluealloy/revm/pull/1614))
- group optimism invalid txn errors ([#1604](https://github.com/bluealloy/revm/pull/1604))
- load_account -> warm_preloaded_addresses ([#1584](https://github.com/bluealloy/revm/pull/1584))
- Refactor code, and check if precompile for create collision ([#1600](https://github.com/bluealloy/revm/pull/1600))
- *(revm)* defer bytecode load ([#1588](https://github.com/bluealloy/revm/pull/1588))
- Rename gas_price to gas_limit for precompile args ([#1593](https://github.com/bluealloy/revm/pull/1593))

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-v10.0.0...revm-v11.0.0) - 2024-07-08

### Added
- add bytecode_address from CallInputs to Contract during construction. ([#1568](https://github.com/bluealloy/revm/pull/1568))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/bluealloy/revm/pull/1565))
- *(EOF)* disallow ExtDelegateCall to legacy bytecode ([#1572](https://github.com/bluealloy/revm/pull/1572))
- *(EOF)* Add target address expansion checks ([#1570](https://github.com/bluealloy/revm/pull/1570))

### Other
- bump precompile to v9.0.0 ([#1590](https://github.com/bluealloy/revm/pull/1590))
- *(README)* add rbuilder to used-by ([#1585](https://github.com/bluealloy/revm/pull/1585))
- Use HandleOrRuntime to allow alloydb/ethersdb to hold a custom runtime ([#1576](https://github.com/bluealloy/revm/pull/1576))
- store tokio::runtime::Handle in ethers/alloyDB ([#1557](https://github.com/bluealloy/revm/pull/1557))
- use const blocks ([#1522](https://github.com/bluealloy/revm/pull/1522))
- fix compile for alloydb ([#1559](https://github.com/bluealloy/revm/pull/1559))
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/bluealloy/revm/pull/1505))

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-v9.0.0...revm-v10.0.0) - 2024-06-20

### Added
- *(revm)* derive serde for `BundleState` ([#1539](https://github.com/bluealloy/revm/pull/1539))
- bump alloy, re-enable alloydb ([#1533](https://github.com/bluealloy/revm/pull/1533))
- mutable access for all fields in BundleBuilder ([#1524](https://github.com/bluealloy/revm/pull/1524))
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/bluealloy/revm/pull/1504))
- add helpers for working with instruction tables ([#1493](https://github.com/bluealloy/revm/pull/1493))
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/bluealloy/revm/pull/1499))
- Persist reverted account and storage slot lookups in `JournaledState` ([#1437](https://github.com/bluealloy/revm/pull/1437))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/bluealloy/revm/pull/1467))
- *(EOF)* Add EOF to inspector handle register ([#1469](https://github.com/bluealloy/revm/pull/1469))
- *(optimism)* Implement new L1 cost function for Fjord ([#1420](https://github.com/bluealloy/revm/pull/1420))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/bluealloy/revm/pull/1436))
- *(revm)* revert EIP-2935 BLOCKHASH opcode changes ([#1450](https://github.com/bluealloy/revm/pull/1450))
- load account should return db error ([#1447](https://github.com/bluealloy/revm/pull/1447))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/bluealloy/revm/pull/1415))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- *(eof)* output gas for eofcreate ([#1540](https://github.com/bluealloy/revm/pull/1540))
- *(revm)* remove storage reset that clears is_cold flag ([#1518](https://github.com/bluealloy/revm/pull/1518))
- *(op)* Remove `U256::from(<float>)` ([#1498](https://github.com/bluealloy/revm/pull/1498))
- *(EOF)* panic on empty input range, and continue exec after eofcreate ([#1477](https://github.com/bluealloy/revm/pull/1477))
- *(Interpreter)* wrong block number used ([#1458](https://github.com/bluealloy/revm/pull/1458))
- blockchash for devnet-0  ([#1427](https://github.com/bluealloy/revm/pull/1427))

### Other
- Add CI build target for no-std + optimism, use matrix builds ([#1551](https://github.com/bluealloy/revm/pull/1551))
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- avoid cloning precompiles ([#1486](https://github.com/bluealloy/revm/pull/1486))
- add setters to `BundleBuilder` with `&mut self` ([#1527](https://github.com/bluealloy/revm/pull/1527))
- pluralize EOFCreateInput ([#1523](https://github.com/bluealloy/revm/pull/1523))
- added simular to used-by ([#1521](https://github.com/bluealloy/revm/pull/1521))
- Removed .clone() in ExecutionHandler::call, and reusing output buffer in Interpreter ([#1512](https://github.com/bluealloy/revm/pull/1512))
- remove old deprecated items ([#1489](https://github.com/bluealloy/revm/pull/1489))
- *(deps)* bump rstest from 0.19.0 to 0.21.0 ([#1482](https://github.com/bluealloy/revm/pull/1482))
- *(deps)* bump tokio from 1.37.0 to 1.38.0 ([#1480](https://github.com/bluealloy/revm/pull/1480))
- *(primitives)* rename State/Storage to EvmState/EvmStorage ([#1459](https://github.com/bluealloy/revm/pull/1459))
- remove 'checked' bytecode bench causing benchmarks to crash due to name ([#1461](https://github.com/bluealloy/revm/pull/1461))
- cargo update ([#1451](https://github.com/bluealloy/revm/pull/1451))
- cleanup host blockhash fn ([#1430](https://github.com/bluealloy/revm/pull/1430))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))" ([#1426](https://github.com/bluealloy/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))
- *(deps)* bump anyhow from 1.0.82 to 1.0.83 ([#1404](https://github.com/bluealloy/revm/pull/1404))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-v8.0.0...revm-v9.0.0) - 2024-05-12

### Added
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bluealloy/revm/pull/1389))
- add a hook to execute individual frames ([#1369](https://github.com/bluealloy/revm/pull/1369))
- *(Handler)* Add ClearHandle ([#1368](https://github.com/bluealloy/revm/pull/1368))
- Add uniswap V2 WETH-USDC swap example ([#1353](https://github.com/bluealloy/revm/pull/1353))
- *(interpreter)* add helpers for spending all gas ([#1360](https://github.com/bluealloy/revm/pull/1360))
- add helper methods to CallInputs ([#1345](https://github.com/bluealloy/revm/pull/1345))
- *(revm)* make `ItemOrResult` serializable ([#1282](https://github.com/bluealloy/revm/pull/1282))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- *(`db`)* Introduce `alloydb` ([#1257](https://github.com/bluealloy/revm/pull/1257))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/bluealloy/revm/pull/1243))
- *(interpreter)* test Host object-safety, allow `dyn Host` in instructions ([#1245](https://github.com/bluealloy/revm/pull/1245))

### Fixed
- *(eip2935)* Preload blockchash storage address ([#1395](https://github.com/bluealloy/revm/pull/1395))
- return the correct error in resize_memory ([#1359](https://github.com/bluealloy/revm/pull/1359))

### Other
- add Trin to used by list ([#1393](https://github.com/bluealloy/revm/pull/1393))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- bump alloy & specify dep rev ([#1380](https://github.com/bluealloy/revm/pull/1380))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))
- *(ci)* bump action/deploy ([#1372](https://github.com/bluealloy/revm/pull/1372))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/bluealloy/revm/pull/1307))
- *(deps)* bump anyhow from 1.0.81 to 1.0.82 ([#1293](https://github.com/bluealloy/revm/pull/1293))
- fix some warnings ([#1305](https://github.com/bluealloy/revm/pull/1305))
- Update documentation ([#1275](https://github.com/bluealloy/revm/pull/1275))
- *(interpreter)* use `pop_top!` where possible ([#1267](https://github.com/bluealloy/revm/pull/1267))
- add and use EvmContext::take_error ([#1264](https://github.com/bluealloy/revm/pull/1264))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-v7.2.0...revm-v8.0.0) - 2024-04-02

### Added
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/bluealloy/revm/pull/1234))

### Fixed
- *(TracerEip3155)* clear Inspector data after transaction. ([#1230](https://github.com/bluealloy/revm/pull/1230))
- *(GasInspector)* calculate correct remaining gas after call return ([#1236](https://github.com/bluealloy/revm/pull/1236))
- fix eip3155 summary gas_used bug and add fork name ([#1216](https://github.com/bluealloy/revm/pull/1216))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))
- *(deps)* bump tokio from 1.36.0 to 1.37.0 ([#1244](https://github.com/bluealloy/revm/pull/1244))
- *(interpreter)* unbox contract field ([#1228](https://github.com/bluealloy/revm/pull/1228))
- *(primitives)* kzg intro ([#1209](https://github.com/bluealloy/revm/pull/1209))
- *(interpreter)* keep track of remaining gas rather than spent ([#1221](https://github.com/bluealloy/revm/pull/1221))
- Improve `EthersDB` ([#1208](https://github.com/bluealloy/revm/pull/1208))

## [7.2.0](https://github.com/bluealloy/revm/compare/revm-v7.1.0...revm-v7.2.0) - 2024-03-19

### Added
- add convert_boxed and insert_boxed for InstructionTable ([#1194](https://github.com/bluealloy/revm/pull/1194))
- optional nonce check ([#1195](https://github.com/bluealloy/revm/pull/1195))

### Other
- Change unwrap to ? to propagate errors ([#1207](https://github.com/bluealloy/revm/pull/1207))
- fix wonky test ([#1197](https://github.com/bluealloy/revm/pull/1197))
- clippy ([#1196](https://github.com/bluealloy/revm/pull/1196))
- *(deps)* bump anyhow from 1.0.80 to 1.0.81 ([#1187](https://github.com/bluealloy/revm/pull/1187))
- fix some typos ([#1189](https://github.com/bluealloy/revm/pull/1189))
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))
- Update post_execution.rs ([#1180](https://github.com/bluealloy/revm/pull/1180))

## [7.1.0](https://github.com/bluealloy/revm/compare/revm-v7.0.0...revm-v8.0.0) - 2024-03-08

### Added
- Restrict ContextPrecompiles only to EvmContext ([#1174](https://github.com/bluealloy/revm/pull/1174))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-v6.1.0...revm-v7.0.0) - 2024-03-08

This release got yanked and replaced with 7.1.0

### Added
- add insert method on instruction table ([#1167](https://github.com/bluealloy/revm/pull/1167))
- precompile with generic context ([#1155](https://github.com/bluealloy/revm/pull/1155))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- add more JournaledState methods to EvmContext ([#1158](https://github.com/bluealloy/revm/pull/1158))
- add example for using a database by reference ([#1150](https://github.com/bluealloy/revm/pull/1150))
- Add boxed precompile trait ([#1131](https://github.com/bluealloy/revm/pull/1131))
- add with_handler method to EvmBuilder ([#1124](https://github.com/bluealloy/revm/pull/1124))

### Fixed
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/bluealloy/revm/pull/1169))
- gas cost calculation ([#1166](https://github.com/bluealloy/revm/pull/1166))
- reset tstorage on finalize ([#1168](https://github.com/bluealloy/revm/pull/1168))
- make `feature = "optional_gas_refund"` work ([#1134](https://github.com/bluealloy/revm/pull/1134))

### Other
- *(deps)* bump ethers-contract from 2.0.13 to 2.0.14 ([#1161](https://github.com/bluealloy/revm/pull/1161))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))
- remove clone for context in handler_register.rs ([#1138](https://github.com/bluealloy/revm/pull/1138))
- Check runtime dynamically ([#1135](https://github.com/bluealloy/revm/pull/1135))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/bluealloy/revm/pull/1132))
- Add `db` and `db_mut` to evm ([#1133](https://github.com/bluealloy/revm/pull/1133))
- add ToString for no_std add exports some types in no_std ([#1128](https://github.com/bluealloy/revm/pull/1128))
- Add `clone` method to `ContextWithHandlerCfg` ([#1127](https://github.com/bluealloy/revm/pull/1127))
- remove unused EvmInstructionTables type alias ([#1123](https://github.com/bluealloy/revm/pull/1123))

## [6.1.0](https://github.com/bluealloy/revm/compare/revm-v6.0.0...revm-v6.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))
- split off serde_json dependency to its own feature ([#1104](https://github.com/bluealloy/revm/pull/1104))

### Fixed
- replace tuple in sstore return with struct ([#1115](https://github.com/bluealloy/revm/pull/1115))
- fix EthersDB deadlock ([#1089](https://github.com/bluealloy/revm/pull/1089))
- Handle fatal db error on load_account ([#1111](https://github.com/bluealloy/revm/pull/1111))

### Other
- Ensure `L1Block` is in the cache ([#1121](https://github.com/bluealloy/revm/pull/1121))
- Fix feature name for generate_block_traces example ([#1120](https://github.com/bluealloy/revm/pull/1120))
- *(refactor)* Propagate fatal error ([#1116](https://github.com/bluealloy/revm/pull/1116))
- Revert "fix EthersDB deadlock ([#1089](https://github.com/bluealloy/revm/pull/1089))" ([#1118](https://github.com/bluealloy/revm/pull/1118))
- Remove DatabaseRef bound on CacheDB ([#1113](https://github.com/bluealloy/revm/pull/1113))
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))
- *(deps)* bump anyhow from 1.0.79 to 1.0.80 ([#1108](https://github.com/bluealloy/revm/pull/1108))
- improve EIP-3155 implementation ([#1105](https://github.com/bluealloy/revm/pull/1105))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-v5.0.0...revm-v6.0.0) - 2024-02-17

### Added
- improve OriginalValuesKnown docs ([#1083](https://github.com/bluealloy/revm/pull/1083))

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))
- modify cfg spec_id ([#1095](https://github.com/bluealloy/revm/pull/1095)) ([#1096](https://github.com/bluealloy/revm/pull/1096))
- optimism compilation ([#1091](https://github.com/bluealloy/revm/pull/1091))

### Other
- *(state)* avoid cloning full account ([#1097](https://github.com/bluealloy/revm/pull/1097))
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- Add memory offset ([#1032](https://github.com/bluealloy/revm/pull/1032))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-v4.0.0...revm-v5.0.0) - 2024-02-12

### Fixed
- properly set context env ([#1070](https://github.com/bluealloy/revm/pull/1070))
- typo on internal append_handle_register methods ([#1069](https://github.com/bluealloy/revm/pull/1069))
- *(op)* skip validation on deposit tx ([#1065](https://github.com/bluealloy/revm/pull/1065))

### Other
- *(precompile)* make use of padding utilities, simplify secp256k1 ([#1073](https://github.com/bluealloy/revm/pull/1073))
- *(revm)* Add helpers to Build Revm with Context ([#1068](https://github.com/bluealloy/revm/pull/1068))
- *(revme)* statetests new format and return error ([#1066](https://github.com/bluealloy/revm/pull/1066))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-v3.5.0...revm-v4.0.0) - 2024-02-07

Refactored the logic inside Handler and added EvmBuilder that allows overwriting the default behavior.
Few major renaming: EVMImpl to Evm, EVM to EvmFactory and EVMData to EvmContext.

### Added
- *(handler)* Change spec id on &mut ([#1055](https://github.com/bluealloy/revm/pull/1055))
- *(Handler)* add push and pop of hanler registers ([#1053](https://github.com/bluealloy/revm/pull/1053))
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- *(inspector)* Share call/create inputs in Inspector call_end/create_end ([#1003](https://github.com/bluealloy/revm/pull/1003))
- Convert optimism panic into graceful error ([#982](https://github.com/bluealloy/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- *(ethersdb)* propagate errors instead of panicking in basic_ref ([#935](https://github.com/bluealloy/revm/pull/935))
- *(revm)* implement prepend_state for BundleState ([#907](https://github.com/bluealloy/revm/pull/907))
- add serde derives for `CacheDB` under "serde" flag ([#911](https://github.com/bluealloy/revm/pull/911))
- *(examples)* generate block traces ([#895](https://github.com/bluealloy/revm/pull/895))
- *(revm)* Evm Context Tests and test-utils Feature ([#903](https://github.com/bluealloy/revm/pull/903))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- transition account balance delta ([#843](https://github.com/bluealloy/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- add more `auto_impl`s to revm traits ([#799](https://github.com/bluealloy/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- add methods to `CreateInput` for calculating created address ([#793](https://github.com/bluealloy/revm/pull/793))
- *(revm)* implement DatabaseRef trait for EthersDB ([#774](https://github.com/bluealloy/revm/pull/774))

### Fixed
- fix previous commit ([#1044](https://github.com/bluealloy/revm/pull/1044))
- *(State)* Preserve original values on delete revert ([#1010](https://github.com/bluealloy/revm/pull/1010))
- optimism gas refunds ([#989](https://github.com/bluealloy/revm/pull/989))
- dont calculate initcode keccak on CREATE ([#969](https://github.com/bluealloy/revm/pull/969))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- safer stack ([#879](https://github.com/bluealloy/revm/pull/879))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/bluealloy/revm/pull/824))
- fix typos in revm crate ([#821](https://github.com/bluealloy/revm/pull/821))
- Optimism execution ([#789](https://github.com/bluealloy/revm/pull/789))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/bluealloy/revm/pull/795))

### Other
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/bluealloy/revm/pull/1059))
- spelling on last commit ([#1058](https://github.com/bluealloy/revm/pull/1058))
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(deps)* bump tokio from 1.35.1 to 1.36.0 ([#1052](https://github.com/bluealloy/revm/pull/1052))
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/bluealloy/revm/pull/1050))
- *(deps)* bump ethers-contract from 2.0.11 to 2.0.13 ([#1034](https://github.com/bluealloy/revm/pull/1034))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/bluealloy/revm/pull/1041))
- group handlers ([#1030](https://github.com/bluealloy/revm/pull/1030))
- *(Inspector)* add inspector depth test ([#1028](https://github.com/bluealloy/revm/pull/1028))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/bluealloy/revm/pull/1026))
- *(clippy)* nightly clippy ([#1025](https://github.com/bluealloy/revm/pull/1025))
- *(Execution)* Granular handles create/call,call_return,insert_call_outcome ([#1024](https://github.com/bluealloy/revm/pull/1024))
- *(Inspector)* Add return_memory_offset to Inspector::call ([#1006](https://github.com/bluealloy/revm/pull/1006))
- update call end docs ([#1000](https://github.com/bluealloy/revm/pull/1000))
- add getter for specId ([#998](https://github.com/bluealloy/revm/pull/998))
- Remove preserve_order in serde_json ([#997](https://github.com/bluealloy/revm/pull/997))
- update create docs ([#999](https://github.com/bluealloy/revm/pull/999))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/bluealloy/revm/pull/994))
- *(Inspector)* add CallOutcome to call/call_end ([#985](https://github.com/bluealloy/revm/pull/985))
- set deduct_caller in optimism handler ([#988](https://github.com/bluealloy/revm/pull/988))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(Inspector)* Add CreateOutcome in create/create_end return ([#980](https://github.com/bluealloy/revm/pull/980))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- *(EvmBuilder)* Remove unnecessary BuilderStage trait ([#979](https://github.com/bluealloy/revm/pull/979))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* refactor sstore_cost ([#974](https://github.com/bluealloy/revm/pull/974))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(deps)* bump anyhow from 1.0.77 to 1.0.79 ([#950](https://github.com/bluealloy/revm/pull/950))
- relax Bytes requirement and use slice instead ([#937](https://github.com/bluealloy/revm/pull/937))
- *(deps)* bump futures from 0.3.29 to 0.3.30 ([#927](https://github.com/bluealloy/revm/pull/927))
- *(deps)* bump anyhow from 1.0.75 to 1.0.76 ([#921](https://github.com/bluealloy/revm/pull/921))
- *(deps)* bump tokio from 1.34.0 to 1.35.0 ([#909](https://github.com/bluealloy/revm/pull/909))
- *(revm)* leverage StorageSlot methods, where appropriate ([#899](https://github.com/bluealloy/revm/pull/899))
- relax state generic ([#881](https://github.com/bluealloy/revm/pull/881))
- clippy ([#877](https://github.com/bluealloy/revm/pull/877))
- *(deps)* bump ethers-contract from 2.0.10 to 2.0.11 ([#867](https://github.com/bluealloy/revm/pull/867))
- bump k256 and use normalize_s ([#870](https://github.com/bluealloy/revm/pull/870))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/bluealloy/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/bluealloy/revm/pull/859))
- *(deps)* bump tokio from 1.33.0 to 1.34.0 ([#856](https://github.com/bluealloy/revm/pull/856))
- change addresses to iterator and add into_addresses ([#855](https://github.com/bluealloy/revm/pull/855))
- use keccak256 for blockhash ([#854](https://github.com/bluealloy/revm/pull/854))
- review safety comments ([#811](https://github.com/bluealloy/revm/pull/811))
- *(deps)* bump futures from 0.3.28 to 0.3.29 ([#839](https://github.com/bluealloy/revm/pull/839))
- *(state)* consistent selfdestruct status transition ([#847](https://github.com/bluealloy/revm/pull/847))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/bluealloy/revm/pull/844))
- *(state)* simplify control flow in `CacheState::apply_evm_state` ([#842](https://github.com/bluealloy/revm/pull/842))
- Refactor precompile list from Hash to vec ([#823](https://github.com/bluealloy/revm/pull/823))
- *(state)* make `State::apply_transition` pub ([#832](https://github.com/bluealloy/revm/pull/832))
- *(state)* make bundle state non-optional ([#828](https://github.com/bluealloy/revm/pull/828))
- Refactor evm data to its file ([#815](https://github.com/bluealloy/revm/pull/815))
- for now support 1.69 rust compiler ([#814](https://github.com/bluealloy/revm/pull/814))
- refactor main return to handle ([#808](https://github.com/bluealloy/revm/pull/808))
- *(SharedMemory)* small refactor; tests ([#806](https://github.com/bluealloy/revm/pull/806))
- use `array::from_fn` in `make_instruction_table` ([#809](https://github.com/bluealloy/revm/pull/809))
- remove `step` and `step_end` return result ([#804](https://github.com/bluealloy/revm/pull/804))
- Instruction table ([#759](https://github.com/bluealloy/revm/pull/759))
- getter for  field of ([#792](https://github.com/bluealloy/revm/pull/792))
- Shared memory between calls ([#673](https://github.com/bluealloy/revm/pull/673))
- Fix typos ([#790](https://github.com/bluealloy/revm/pull/790))
- *(deps)* bump tokio from 1.32.0 to 1.33.0 ([#785](https://github.com/bluealloy/revm/pull/785))
- Use upstream create and create2 implementations ([#775](https://github.com/bluealloy/revm/pull/775))
- reorder JournalState impl ([#772](https://github.com/bluealloy/revm/pull/772))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v3.5.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* 4e78fbe - docs: document non-zero amounts in State::increment_balances (#760) (15 hours ago) <Dan Cline>
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>

# v3.4.0
date: 28.09.2023

Summary:
* Cancun ready. all EIP implemented.
  Check interpreter CHANGELOG
* revm State. a `Database` that handles Reverts and state transitions.
* Optimism support
* no_std build

Note: c-kzg can't be build for wasm and is behind "c-kzg" feature flag.

Full git log:
* ea0d8d8 - fix: use u128 for calc data fee result (#757) (46 minutes ago) <Dan Cline>
* 4f916be - chore: bump c-kzg to create lib (#758) (5 hours ago) <rakita>
* ded673c - docs: Readme Updates (#756) (16 hours ago) <refcell.eth>
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d2a066b - ci: concurrency for github actions (#750) (25 hours ago) <Paul Razvan Berg>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 2c556c0 - refactor: say "warm" instead of "hot" (#754) (25 hours ago) <Paul Razvan Berg>
* 8a85d19 - fix: balance check disabled (#751) (25 hours ago) <Wodann>
* b9938a8 - chore(deps): bump sha2 from 0.10.7 to 0.10.8 (#752) (30 hours ago) <dependabot[bot]>
* 4829e6a - chore(deps): bump thiserror from 1.0.48 to 1.0.49 (#753) (30 hours ago) <dependabot[bot]>
* 8206193 - feat: add "kzg" as a separate feature (#746) (3 hours ago) <DaniPopes>
* 4b5fa61 - EIP-6780: SELFDESTRUCT only in same transaction (#719) (5 days ago) <Lorenzo Feroleto>
* f72eaa0 - chore: error type for block header (#731) (5 days ago) <hack3r-0m>
* cb39117 - fix(eip4844): Pass eth tests, additional conditions added. (#735) (6 days ago) <rakita>
* c2cde03 - fix: use CANCUN precompile id for CANCUN SpecId (#733) (6 days ago) <Dan Cline>
* d926728 - perf: refactor interpreter internals and cleanup (#582) (6 days ago) <DaniPopes>
* 1b8cd57 - make BundleBuilder publicly available (#729) (8 days ago) <Thomas Coratger>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 9f00e37 - feat(state): remove state sorting, no_std ci,remove rayon (#717) (13 days ago) <rakita>
* 429da73 - chore(perf): only recalc code hash if its the default (#716) (13 days ago) <evalir>
* e2ecd5e - docs: add warning on panic conditions in take_bundle (#715) (2 weeks ago) <Dan Cline>
* 190f90e - Never inline the prepare functions (#712) (2 weeks ago) <Valentin Mihov>
* 26dc07d - feat: return wiped inside storage changeset (#711) (2 weeks ago) <rakita>
* 5d68dd5 - chore(deps): bump bytes from 1.4.0 to 1.5.0 (#707) (2 weeks ago) <dependabot[bot]>
* fd8d4c5 - chore(deps): bump ethers-contract from 2.0.9 to 2.0.10 (#705) (2 weeks ago) <dependabot[bot]>
* e86c19b - chore(state): do not insert empty reverts in state (#702) (3 weeks ago) <Lorenzo Feroleto>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 1d6a039 - chore: make `impl Default for StateBuilder` generic (#690) (3 weeks ago) <DaniPopes>
* c60abcf - feat(state): Nits, builder option and OriginalValueKnown flags (#699) (3 weeks ago) <rakita>
* 7e7a339 - bundle size hint (#670) (3 weeks ago) <Roman Krasiuk>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* d04aad3 - chore: expose StateDBBox (#694) (3 weeks ago) <rakita>
* ee13aac - feat(StateBuilder): switch builder option from without_bundle to with_bundle (#688) (3 weeks ago) <rakita>
* 7d7f63f - chore(state): Make Database more generic. (#687) (3 weeks ago) <rakita>
* a9dce30 - chore: nits and renamings (#684) (3 weeks ago) <rakita>
* b500718 - feat(state): take N reverts from BundleState, struct refactor (#681) (3 weeks ago) <rakita>
* fde6df1 - apply builder pattern for BundleState initialization (#649) (3 weeks ago) <Eric>
* 2897655 - fix(state): Extend now properly transfers wiped storage (#675) (3 weeks ago) <rakita>
* 6bd05c9 - chore: impl Eq, PartialEq for TransitionState (#677) (4 weeks ago) <Dan Cline>
* 175aaec - Removed the last dependencies breaking no-std build. (#669) (4 weeks ago) <Lucas Clemente Vella>
* 4272535 - fix(state): retain destroyed account status on bundle extend (#667) (4 weeks ago) <rakita>
* bef9edd - chore(state): bundle retention (#666) (4 weeks ago) <Roman Krasiuk>
* 1053d0e - fix(state): Regresion, remove present info on selfdestruct (#664) (4 weeks ago) <rakita>
* 6c4cd31 - feat: add BundleState::revert_latest (#661) (4 weeks ago) <Matthias Seitz>
* fd2cc88 - fix(state): state transition regression (#662) (4 weeks ago) <Roman Krasiuk>
* c14f8a9 - feat(state): add a flag allowing transition merge without reverts (#657) (4 weeks ago) <Roman Krasiuk>
* 33498d7 - chore(deps): bump ethers-contract from 2.0.8 to 2.0.9 (#640) (4 weeks ago) <dependabot[bot]>
* 9a88c99 - chore: filter out empty bytecode from bundle (#656) (4 weeks ago) <rakita>
* 98a4a18 - feat(state): Make Bundle extend wipe aware (#655) (4 weeks ago) <rakita>
* 1bf0315 - feat(state): ability to disable reverts collection in bundle state (#654) (4 weeks ago) <Roman Krasiuk>
* 3eea324 - fix(state): drop storage only for DestroyedChanged (#651) (4 weeks ago) <rakita>
* 37027db - fix revert from DestroyedChanged to DestroyedAgain (#648) (5 weeks ago) <rakita>
* cec7f82 - chore(state): bundle state split (#646) (5 weeks ago) <Roman Krasiuk>
* ff5a2bc - add value parameter to Inspector::selfdestruct (#645) (5 weeks ago) <Tony Ke>
* b2d6f7a - Refactor: Split transaction pre verification to separate function (#573) (5 weeks ago) <Lorenzo Feroleto>
* afbc896 - fix(state): check if storage revert is empty (#643) (5 weeks ago) <Roman Krasiuk>
* 0b9c12e - test(state): bundle selfdestructs (#627) (5 weeks ago) <Roman Krasiuk>
* 6b55b9c - feat(`interpreter`): add hash to bytecode (#628) (5 weeks ago) <evalir>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 43d535c - style: bundle state (#637) (5 weeks ago) <Roman Krasiuk>
* f843592 - fix(state): return RevertToSlot struct with more info (#636) (5 weeks ago) <rakita>
* aee1d1c - bug(state): remove redundunt info revert on destruct (#635) (5 weeks ago) <rakita>
* 321152a - book workflow (#537) (5 weeks ago) <Waylon Jepsen>
* 0028193 - feat: Optional coinbase tip (#625) (5 weeks ago) <clabby>
* 6ea1edc - test(state): bundle reverts collapse (#626) (5 weeks ago) <Roman Krasiuk>
* a40f272 - feat(state): Use preloaded bundle inside state (#622) (5 weeks ago) <rakita>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* e5227c4 - test(state): account & storage revert value preservation (#614) (6 weeks ago) <Roman Krasiuk>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* 588503a - chore: get or insert bundle state (#613) (6 weeks ago) <Roman Krasiuk>
* 7e83c7f - fix(inspector): call call_end/create_end when inspector shortcircuits calls (#609) (6 weeks ago) <evalir>
* adf42b2 - chore(deps): bump anyhow from 1.0.74 to 1.0.75 (#606) (6 weeks ago) <dependabot[bot]>
* 0e85fdf - chore(deps): bump tokio from 1.31.0 to 1.32.0 (#607) (6 weeks ago) <dependabot[bot]>
* 449d6b9 - chore: export some `unreachable_pub` items (#598) (6 weeks ago) <DaniPopes>
* 5d0b54d - chore(deps): bump anyhow from 1.0.72 to 1.0.74 (#602) (6 weeks ago) <dependabot[bot]>
* c785115 - fix: Load caller in safe way in finalization fn (#604) (6 weeks ago) <rakita>
* dfae7fe - chore: fix test build, use new types (#605) (6 weeks ago) <rakita>
* fc2107c - chore: Revert test, not change storage check , renaming of original slot value (#601) (6 weeks ago) <rakita>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* 49a6470 - chore: `TransitionState::with_capacity` -> `TransitionState::single` (#600) (6 weeks ago) <Roman Krasiuk>
* f4224d8 - perf: pre-allocate inner bundle state (#599) (6 weeks ago) <Roman Krasiuk>
* 5cdaa97 - chore: avoid unnecessary allocations (#581) (6 weeks ago) <DaniPopes>
* da26d0d - chore(deps): bump tokio from 1.29.1 to 1.31.0 (#595) (6 weeks ago) <dependabot[bot]>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 1478724 - chore: move precompiles to EVMData for inspectors (#588) (7 weeks ago) <evalir>
* fe6c54e - fix(transient_storage): set previous value in journal (#585) (7 weeks ago) <rakita>
* bd84a07 - refactor: rewrite revm-test as a criterion bench (#579) (7 weeks ago) <DaniPopes>
* 5734f12 - fix: AccessList with two same addresses (#578) (8 weeks ago) <rakita>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* 4686cb3 - fix(revm): EIP-3155 tracer tx output without debug artefact (#552) (9 weeks ago) <Perama>
* 26126ad - fix(revm): extra return in EIP3155 inspector (#563) (9 weeks ago) <Perama>
* 3f6052c - fix(revm): include CREATE/CREATE2 in EIP3155 inspector (#562) (9 weeks ago) <Perama>
* 5ce9dc9 - chore: clippy and fmt (#568) (9 weeks ago) <rakita>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>
* 10f81ba - optimize stack usage for recursive `call` and `create` programs (#522) (3 months ago) <Valentin Mihov>
* 323370a - fix comment (#529) (3 months ago) <Ethan-000>
* 51072e6 - consume all gas on invalid opcode (#500) (3 months ago) <teddav>
* 63f9460 - chore(deps): bump auto_impl from 1.0.1 to 1.1.0 (#478) (3 months ago) <dependabot[bot]>
* 3a77ee5 - docs: fix comment typo (#517) (3 months ago) <Sabnock>
* d343858 - fix: typo in eip-3155 output (#497) (4 months ago) <Perama>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* c3b0312 - docs: add some CacheDB docs (#484) (5 months ago) <Matthias Seitz>
* c81acc6 - feat: Create account checkpoint (#483) (5 months ago) <rakita>
* 6057cc2 - chore: refactor interpreter run and remove static flag (#481) (5 months ago) <rakita>
* d193418 - chore: Bundle inspector crate/call calls (#480) (5 months ago) <rakita>
* 75a6136 - feat: Introduce account status as bitflag inside JournalState (#477) (5 months ago) <rakita>


# v3.3.0
date: 03.05.2023

Consensus bug:
* cde2f2d - fix: revert of selfdestruct with same target address (#475) (2 hours ago) <Roman Krasiuk>

Other small changes:
* bd0fad8 - (HEAD -> reles, origin/main, origin/HEAD) chore(deps): bump tokio from 1.27.0 to 1.28.0 (#470) (52 minutes ago) <dependabot[bot]>
* ccefbca - chore(deps): bump ruint from 1.7.0 to 1.8.0 (#465) (52 minutes ago) <dependabot[bot]>
* 7c2e0f5 - chore(deps): bump anyhow from 1.0.70 to 1.0.71 (#471) (53 minutes ago) <dependabot[bot]>
* d7adfd5 - Fix typo in primitives/src/state.rs (#474) (53 minutes ago) <Udoagwa Franklin>
* d0cd897 - add example to revm crate (#468) (8 days ago) <Sambhav>
* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v3.2.0
date: 19.04.2023

consensus bug:
* fix: touched account on creation (#463) (2 hours ago) <Roman Krasiuk>

# v3.1.1
date: 14.04.2023

bump revm dependency versions.

# v3.1.0
date: 04.04.2022

Main changes can be summarizes in:
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
    * removal of gas block allowed us to have more compact analysis data. Gas block from beginning didn't have big impact on performance but introduced not intuitive gas calculations that was
    source of some bugs. 
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
    * revm is Shanghai ready
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
    * There wasone  bug inside CacheDB that was here for a long time, and would happen only if
    selfdestruct/create2 is called in multiple transaction on same account on same cache data.
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>


Changelogs:
* 9edb8f4 - (origin/main, origin/HEAD) Improve EthersDB::new (#440) (5 days ago) <lazymio>
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 6b09caf - chore(deps): bump serde_json from 1.0.94 to 1.0.95 (#434) (6 days ago) <dependabot[bot]>
* 77f1735 - chore(deps): bump walkdir from 2.3.2 to 2.3.3 (#426) (8 days ago) <dependabot[bot]>
* ed981c3 - chore(deps): bump serde from 1.0.157 to 1.0.158 (#425) (8 days ago) <dependabot[bot]>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* 2d5b710 - Comment Fix (#430) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* d935525 - chore(deps): bump secp256k1 from 0.26.0 to 0.27.0 (#429) (2 weeks ago) <dependabot[bot]>
* a85ff79 - Update README.md (#424) (2 weeks ago) <Waylon Jepsen>
* 9645015 - chore(deps): bump thiserror from 1.0.38 to 1.0.40 (#421) (2 weeks ago) <dependabot[bot]>
* aa6519f - chore(deps): bump enumn from 0.1.6 to 0.1.8 (#422) (2 weeks ago) <dependabot[bot]>
* d63146f - chore(deps): bump futures from 0.3.26 to 0.3.27 (#416) (2 weeks ago) <dependabot[bot]>
* 52fe7c4 - chore(deps): bump serde_json from 1.0.93 to 1.0.94 (#401) (2 weeks ago) <dependabot[bot]>
* b98d9c9 - chore(deps): bump serde from 1.0.152 to 1.0.157 (#423) (2 weeks ago) <dependabot[bot]>
* 3d8ca66 - feat: add Output::into_data (#420) (3 weeks ago) <Matthias Seitz>
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* b1208fe - feat: add contract+target to selfdestruct hook (#410) (4 weeks ago) <Matthias Seitz>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* f2656b7 - chore: add primitive SpecId to precompile SpecId conversion (#408) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* cff1070 - Update readmdoc of `perf_analyse_created_bytecodes` (#404) (4 weeks ago) <rakita>
* fbc62a3 - chore: fix typo StorageChange (#403) (4 weeks ago) <Matthias Seitz>
* 7bb73da - feat: Add check for chainID (#393) (4 weeks ago) <chirag-bgh>
* 3a17ca8 - feat: add b256<->u256 from impls (#398) (4 weeks ago) <Matthias Seitz>
* 3789509 - feat: add API to retrieve unpadded bytecode (#397) (5 weeks ago) <Wodann>
* 5ab154a - chore(deps): bump tokio from 1.25.0 to 1.26.0 (#395) (5 weeks ago) <dependabot[bot]>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* 8dc024a - Add copyright start year (#387) (5 weeks ago) <Marius Kjærstad>
* 4d2f074 - feat: add EVM::with_env (#385) (6 weeks ago) <Matthias Seitz>
* 5efd9d1 - impl NonceTooHigh/ NonceTooLow checks (#383) (6 weeks ago) <gd>
* 8e6f4f2 - chore: fix compilation if serde not enabled (#381) (7 weeks ago) <rakita>
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>
* ec582a8 - chore(deps): bump once_cell from 1.17.0 to 1.17.1 (#378) (7 weeks ago) <dependabot[bot]>
* 188dacf - improvement: derive Debug for DatabaseComponentError (#377) (7 weeks ago) <Wodann>
* 0401cfd - Add B160/B256 From primitive_types traits (#380) (7 weeks ago) <Francesco Cinà>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* d5ebdb0 - chore(deps): bump tokio from 1.24.2 to 1.25.0 (#352) (7 weeks ago) <dependabot[bot]>
* ebaccca - chore(deps): bump futures from 0.3.25 to 0.3.26 (#353) (7 weeks ago) <dependabot[bot]>
* 5788340 - chore(deps): bump bytes from 1.3.0 to 1.4.0 (#355) (7 weeks ago) <dependabot[bot]>
* d3fba88 - chore(deps): bump serde_json from 1.0.92 to 1.0.93 (#365) (7 weeks ago) <dependabot[bot]>
* e22c3f3 - fix: call create_end for all code paths (#362) (7 weeks ago) <Wodann>
* b4c62e9 - chore: rename Then to Than (#368) (7 weeks ago) <Matthias Seitz>
* 1c3e9e3 - improvement: use alloc & core for Arc impl (#367) (8 weeks ago) <Wodann>
* 3158ce9 - feat: implement Debug for DatabaseComponentError if supported (#363) (8 weeks ago) <Wodann>
* d9727c2 - improvement: add error details to InvalidTransaction::LackOfFundForGasLimit (#364) (8 weeks ago) <Wodann>
* 6b170b4 - Use gas price in place of effective gas price for initial balance check (#359) (8 weeks ago) <gd>
* 5d6ecd0 - improvement: implement BlockHash for Arc<BlockHashRef> (#361) (8 weeks ago) <Wodann>
* ae9baba - improvement: implement State for Arc<StateRef> (#360) (8 weeks ago) <Wodann>
* 2e4e800 - chore(deps): bump serde_json from 1.0.91 to 1.0.92 (#357) (8 weeks ago) <dependabot[bot]>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>
* 10187ed - data change (9 weeks ago) <rakita>

# v3.0.0
date 29.01.2022

This is big release that has core changes that breaks compatibility. In summary:
*  Project is refactored into `revm-primitives`,`revm-precompile`,`revm-interpreter` and `revm` to have more flexibility and separation of concerns. And include paths in revm reflect that. So try to find include as `revm::primitives` or `revm::interpreter`
* Parity `primitive-types` was replaced with `ruint` for big numbers and subset of macros are used for native `B160`/`B256` types. 
* Interpreter instructions are unified and now all of them have same signature.
* web3 db was replaces with ethers alternative.
* revmjs lib was removed from crates.
* `revm_precompiles` was renamed to `revm-precompile.`

* Return types are made to have more insight of what have happened inside revm.
* Snailtracer benchmark got around 20% faster.

Github Changelog:
* dc9818f - (HEAD -> o/bump, origin/bump_v20) Bump v20 (13 hours ago) <rakita>
* 75ef0f1 - (origin/main, origin/HEAD) feat: Staticcall internal return (#349) (13 hours ago) <rakita>
* 0194b37 - (t) fix bug introduced in last commit (13 hours ago) <rakita>
* 7b00f32 - Cleanup imports (#348) (14 hours ago) <rakita>
* c14d7ea - fix: enable the examples to run with the current revm (#347) (16 hours ago) <flyq>
* 329fd94 - Wrap all calls to interpreter.gas.erase_cost with checks if USE_GAS is enabled (#346) (2 days ago) <christn>
* 72355f4 - improvement: add logs & return value to revert (#343) (3 days ago) <Wodann>
* 142a1c9 - expose hashbrown::HashMap in primitives (#345) (3 days ago) <Andy Thomson>
* ba393d7 - fix: disable balance check (#342) (4 days ago) <Wodann>
* 876fad1 - refactor: simplify DatabaseComponentError (#339) (6 days ago) <Wodann>
* 81534ad - chore: includes to libs (#338) (7 days ago) <rakita>
* e2f4d32 - Creating revm-primitives, revm better errors and db components  (#334) (10 days ago) <rakita>
* de83db6 - fix: feature flags (#330) (2 weeks ago) <Wodann>
* b60269c - `revm`: mark `with-serde` feature as deprecated (#328) (2 weeks ago) <Enrique Ortiz>
* 63bf475 - make load_account pub (#325) (3 weeks ago) <rakita>
* 0ef0197 - Cleanup, move hot fields toggether in Interpreter (#321) (3 weeks ago) <rakita>
* 81942d6 - enable proptest with arbitrary feature (#323) (3 weeks ago) <joshieDo>
* 2be3798 - feat: revm-interpreter created (#320) (3 weeks ago) <rakita>
* 7e98fef - fix: feature flag compiler errors (#256) (5 weeks ago) <Wodann>
* 488ef8a - Add example for fork + ref_transact impl (#296) (6 weeks ago) <0xDmtri>
* 56e6c22 - feat: allow disabling of balance checks (#297) (6 weeks ago) <Wodann>
* 8661467 - feat: Export CustomPrinter insector from revm (#300) (6 weeks ago) <rakita>
* 222b8e9 - feature: substitute web3db to ethersdb (#293) (6 weeks ago) <0xDmtri>
* fd01083 - feature(revm): Return `bytes` in Create calls (#289) (7 weeks ago) <Nicolas Gotchac>
* 2fb0933 - docs: Correct typo (#282) (7 weeks ago) <Przemyslaw Rzad>
* 90fe01e - feat(interpreter): Unify instruction fn signature (#283) (7 weeks ago) <rakita>
* 54e0333 - bug: Integer overflow while calculating the remaining gas in GasInspector (#287) (8 weeks ago) <rakita>
* acdbaac - native bits (#278) (8 weeks ago) <rakita>
* 69e302b - feat(revm): Add prevrandao field to EnvBlock (#271) (2 months ago) <rakita>
* d1703cd - Export StorageSlot (#265) (3 months ago) <Francesco Cinà>
* 560bb03 - Fix: typos (#263) (3 months ago) <HAPPY>
* 369244e - feat(refactor): make keccak in one place. (#247) (3 months ago) <rakita>
* c96c878 - feat: Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239) (3 months ago) <Alexey Shekhirin>


# v2.3.1
date: 22.11.2022

Bump dependency versions.


# v2.3.0
date: 16.11.2022
Very small release. Exposes one field and added prevrandao to remove footgun of forgeting to set difficulty.

* 927d16c - disable gas refunds with env flag (#267) (14 minutes ago) <gd>
* 47a8310 - Add prevrandao field to EnvBlock (3 minutes ago) <rakita>
* 2c45b04 - Export StorageSlot (#265) (23 minutes ago) <Francesco Cinà>

# v2.2.0
date: 12.11.2022

Small release that contains consensus bug fix. Additionally added few small feature flags needed for hardhat, opcode utility function and removal of web3db block number check. 

* dc3414a - Added OEF spec for tests. Skip HighGasPrice (4 minutes ago) <rakita>
* f462f9d - Bugfix: if returndatacopy is len 0 return after initial cost (#259) (4 minutes ago) <gd>
* ea2f2a2 - fix web3db sanity check (#245) (12 days ago) <Wulder>
* 9f8cdbd - feat: allow block gas limit to be toggled off (#238) (3 weeks ago) <Wodann>
* efd9afc - feat: allow eip3607 to be toggled off (#237) (3 weeks ago) <Wodann>
* 88c72a7 - fix: return out of gas code for precompiled contracts (#234) (3 weeks ago) <Wodann>
* 30462a3 - Fix: typos (#232) (3 weeks ago) <omahs>
* 9f513c1 - Borrow self and add derive traits for OpCode (#231) (4 weeks ago) <Franfran>

# v2.1.0
date: 25.09.2022

GasInspector added by Alexey Shekhirin and some helper functions.
Changes:

* ca14d61 - gas inspector (#222) (7 days ago) <Alexey Shekhirin>
* 1e25c99 - chore: expose original value on storageslot (#216) (13 days ago) <Matthias Seitz>
* aa39d64 - feat: add Memory::shrink_to_fit (#215) (13 days ago) <Matthias Seitz

# v2.0.0
date: 10.09.2022

Release with `Database` interface changed, execution result, consensus bug fixes and support for all past forks. Additional optimizations on evm initialization.

Main changes:
* Add support for old forks. (#191) (9 days ago)
* revm/evm: Return `ExecutionResult`, which includes `gas_refunded` (#169) (4 weeks ago) <Nicolas Gotchac>
* JournaledState (#175)
    * Optimize handling of precompiles. Initialization and account loading.
    * Fixes SELFDESTRUCT bug.
* Optimize calldataload. Some cleanup (#168)
* Handle HighNonce tests (#176)
* feat: expose hash on `BytecodeLocked` (#189) (12 days ago) <Bjerg>
* revm: Update account storage methods in CacheDB (#171) (4 weeks ago) <Nicolas Gotchac>
* reexport revm_precompiles as precompiles (#197) (6 days ago) <Matthias Seitz>
* chore(ci): use ethtests profile for CI tests (#188) (2 weeks ago) <Alexey Shekhirin>
* Bump dependencies version
* current_opcode fn and rename program_counter to instruction_pointer (#211)
* Cfg choose create analysis, option on bytecode size limit (#210)
* Cleanup remove U256 and use u64 for gas calculation (#213)

Consensus bugs:
* SELFDESTRUCT was not handled correctly. It would remove account/storage but it should just mark it for removal. This bug was here from earlier version of revm. (#175)
* fix: set gas_block to empty bytecode (#172). Introduced in v1.8.0 with bytecode format.

# v1.9.0
date: 09.08.2022

Small release. Optimizations

* Cache bytecode hash
* Move override_spec config from Inspector to cfg

# v1.8.0
date: 01.08.2022

Medium release, good performance boost. Database trait has changed to support Bytecode.

* Introduce Bytecode format (#156)
* Update readme files.
* Merge eth/tests supported.

# v1.7.0
date: 11.06.2022

small release:
* Make CacheDB field pub and add few utility functions
* Rename Byzantine to Byzantium

# v1.6.0
date: 02.06.2022

Most changes are relayed to CacheDB and how it saved accounts.

* Introduce account `Touched/Cleared/None` state in CacheDB
* Add missing inspectors `call_end` calls
* bump dependencies and few standard derives.

# v1.5.0
date: 09.06.2022

Consensus error related to gas block optimization and `sstore` min stipend. Solution is to make `sstore` instruction as `gas_block_end` as to not spend future instruction gas when checking min stipend condition introduced in EIP-2200.

* Consensus error with gas block for SSTORE stipend check (#124)
* enable EIP2200 in Istanbul (#125)

# v1.4.1
date: 06.06.2022

Small release:
* chore: export evm_inner (#122)

# v1.4.0
date: 03.06.2022

Small release:
* fix: BLOCKHASH should return 0 if number not in last 256 blocks (#112)
* feat: add getters for cachedb (#119)
* bump some lib versions.

# v1.3.1
date: 11.4.2022

Small fixes release.
* Empty keccak constant and remove access_list.clone (#111)
* chore: typo fixes
* fix is_static for Inspector initialize_interp

# v1.3.0
date: 30.4.2022

There are a lot of big changes that are included in this release as revm was integrated inside foundry.

* A lot of changed on Inspector, added new calls and flushed out how it should be called. Big effort mostly driven by Oliver Nordbjerg
* Big internal refactor and renaming: Machine->Inspector, call/create info are now in structs.
* feat: add serde support to model types. Thank you Matthias Seitz
* Added rust feature that sets memory limit on interpreter that is configurable with env.cfg. by Oliver Nordbjerg.
* Library bumped to higher version.

# v1.2.0
date 20.1.2022

Changes:
* Bump revm_precompile and added new feature for k256 lib.

# v1.1.0
date: 14.1.2022

There is bug introduced in last release with gas blcok optimization, it will crash revm if anywhere in contract is unknown OpCode. And now returning log after execution (ups) included them in eth/tests verification.

Changes:
* Bug fix for unknown OpCode
* Omit edgecase high nonce test. tracer gas fix 
* Some internal cleanup

# v1.0.0
date: 18.12.2021

It feel's like that the lib is in the state that is okay to promote it to the v1 version. Other than that, a lot of optimizations are done and the inspector trait was rewritten.

Changes: 
*  web3 db
*  precalculated gas blocks. Optimization
*  PC opcode as pointer. Optimization
*  U256 div_rem optimization
*  Inspector refactored and it is now closer to Host interface.

Optimization thread: https://github.com/bluealloy/revm/issues/7


# v0.5.0
date: 17.11.2021

A lot of optimization on machine(Interpreter) part, it is now at least 3x faster. On interface side, Error enum was renamed to Return and it is simplified. Additionally if needed gas measuring can be removed with rust feature.

Changes: 
* push instruction optimized.
* mload/mstore and memory optimized
* Gas calculation optimized
* optimize i256
* switch stacks from H256 with U256
* Error's refactor to Return
* clippy/warnings/fmt cleanup
* Bump auto_impl to v0.5
* opcode renaming
* Gas measurment can be removed with rust features.

# v0.4.1
date: 02.11.2021

Change in interface and how you can call evm. There is now multiple Database traits for use and inspector is taken on transact call as reference.

* 20ac70b - Database traits made useful.
* 46b5bcd - EVM Interface changed. Inspector called separately.


# v0.3.1
date: 27.10.2021

remove some warnings for unused imports and done cargo fmt.
# v0.3.0
date: 27.10.2021

Interface revamped and now looks a lot better.

Log:
* 1b1ebd8 - [revm] Interface. Inspector added, Env cleanup. revm-test passes (9 hours ago) <rakita>
* 351d4e0 - BIG interface change (11 hours ago) <rakita>
* a723827 - no_sdt to no_std (2 days ago) <rakita>
* a449bed - [precompiles] spelling, small cleanup (2 days ago) <rakita>


# v0.2.2

Same as v0.2.1 but added readme.
# v0.2.1
date: 25.10.2021

Big refactor, cleanup changes, and updating tests. EIP-3607 added.

Log:
* a6e01de - BIG reorg. workspace added. revm-precompile lib (20 minutes ago) <rakita>
* e50f6d3 - Move merkle trie from revm to eth/tests crate (4 hours ago) <rakita>
* 633ffd4 - Bump tests to v10.1 (28 hours ago) <rakita>
* 14b3de1 - Payment overflow check (30 hours ago) <rakita>
* 6e964ba - EIP-3607: Reject transactions from senders with deployed code (30 hours ago) <rakita>


# v0.2.0
date: 23.10.2021:

Published v0.2.0, first initial version of code. London supported and all eth state test are 100% passing or Istanbul/Berlin/London.


### 17.10.2021:
-For past few weeks working on this structure and project in general become really good and I like it. For me it surved as good distraction for past few weeks and i think i am going to get drained if i continue working on it, so i am taking break and i intend to come back after few months and finish it.
- For status:
    * machine/spec/opcodes/precompiles(without modexp) feels good and I probably dont need to touch them.
    * inspector: is what i wanted, full control on insides of EVM so that we can control it and modify it. will probably needs to add some small tweaks to interface but nothing major.
    * subroutines: Feels okay but it needs more scrutiny just to be sure that all corner cases are covered.
    * Test that are failing (~20) are mostly related to EIP-158: State clearing. For EIP-158 I will time to do it properly.
    * There is probably benefit of replaing HashMap hasher with something simpler, but this is research for another time.
## Project structure: