//! Give-backs to the reservoir alone (`Cfg::reservoir_only_give_backs`) and the legacy share of a
//! state charge, through the mainnet handler on Amsterdam.
//!
//! With the switch off and every legacy share zero, which is the default, the handler is
//! EIP-8037's: the rest of this crate's tests run that way. These pin what the switch and the
//! shares change, each against the same transaction without them.

use bytecode::opcode::{
    CALL, CODECOPY, CREATE, GAS, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH32, PUSH4, RETURN,
    REVERT, SELFDESTRUCT, SSTORE, STOP,
};
use context::{
    result::{EVMError, ExecutionResult, HaltReason},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::{cfg::GasId, Cfg, ContextTr, Database};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{interpreter_action::FrameInit, GasTracker, InstructionResult};
use primitives::{address, eip8037, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{
    instructions::EthInstructions, post_execution, EthPrecompiles, EvmTr, FrameResult, Handler,
    ItemOrResult, MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode};
use std::cell::Cell;

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;

/// Contract the transaction calls.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// Contract [`CONTRACT`] calls.
const CHILD: Address = address!("0x000000000000000000000000000000000000c41d");
/// An address the state holds no account for.
const EMPTY: Address = address!("0x000000000000000000000000000000000000e3e3");

/// Regular gas budget of a transaction; the gas limit above it is the reservoir.
const REGULAR_CAP: u64 = 2_000_000;
/// A reservoir that pays every state charge here.
const RESERVOIR: u64 = 1_000_000;

/// State gas of a fresh slot.
const SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
/// State gas of a new or created account.
const NEW_ACCOUNT: u64 = eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;

/// The legacy shares a chain that keeps its pre-Amsterdam prices in regular gas would set.
const SLOT_SHARE: u64 = 10_000;
const CALL_SHARE: u64 = 25_000;
const SELFDESTRUCT_SHARE: u64 = 16_000;
const CREATE_SHARE: u64 = 20_000;

/// Bytecode under construction.
#[derive(Default)]
struct Code(Vec<u8>);

impl Code {
    fn op(mut self, op: u8) -> Self {
        self.0.push(op);
        self
    }

    /// Pushes `value` as a four-byte word, so the gas the code costs does not depend on it.
    fn push(mut self, value: u64) -> Self {
        let value = u32::try_from(value).expect("test amounts fit in four bytes");
        self.0.push(PUSH4);
        self.0.extend_from_slice(&value.to_be_bytes());
        self
    }

    fn push_address(mut self, address: Address) -> Self {
        self.0.push(PUSH20);
        self.0.extend_from_slice(address.as_slice());
        self
    }

    /// Writes `value` to slot zero.
    fn sstore(self, value: u64) -> Self {
        self.push(value).push(0).op(SSTORE)
    }

    /// `CALL(gas, to, value, 0, 0, 0, 0)`, dropping the success flag; `gas` of `None` forwards
    /// all the call may.
    fn call(self, gas: Option<u64>, to: Address, value: u64) -> Self {
        let code = self
            .op(PUSH0)
            .op(PUSH0)
            .op(PUSH0)
            .op(PUSH0)
            .push(value)
            .push_address(to);
        let code = match gas {
            Some(gas) => code.push(gas),
            None => code.op(GAS),
        };
        code.op(CALL).op(POP)
    }

    /// `CREATE` with [`initcode`], which this code carries after its last instruction.
    fn create_and_stop(self) -> Self {
        let initcode = initcode();
        let len = initcode.len() as u64;
        let with_offset = |offset| {
            Code(self.0.clone())
                .push(len)
                .push(offset)
                .op(PUSH0)
                .op(CODECOPY)
                .push(len)
                .op(PUSH0)
                .op(PUSH0)
                .op(CREATE)
                .op(POP)
                .op(STOP)
        };
        // Four-byte pushes keep the prefix length independent of the offset it carries.
        let offset = with_offset(0).0.len() as u64;
        let mut code = with_offset(offset);
        code.0.extend_from_slice(&initcode);
        code
    }

    fn revert(self) -> Self {
        self.op(PUSH0).op(PUSH0).op(REVERT)
    }

    fn build(self) -> Bytecode {
        Bytecode::new_raw(Bytes::from(self.0))
    }
}

/// Initcode that deploys one `STOP` byte.
fn initcode() -> Vec<u8> {
    vec![PUSH1, STOP, PUSH0, MSTORE, PUSH1, 1, PUSH1, 31, RETURN]
}

/// A frame's gas as it returned, before its caller settled it.
#[derive(Clone, Copy, Debug)]
struct Returned {
    result: InstructionResult,
    gas: GasTracker,
}

/// The mainnet handler, recording gas as frames return and settle.
#[derive(Default)]
struct Observer {
    returned: Vec<Returned>,
    settled: Cell<Option<GasTracker>>,
}

impl Observer {
    fn record(&mut self, result: &FrameResult) {
        self.returned.push(Returned {
            result: result.instruction_result(),
            gas: *result.gas().tracker(),
        });
    }
}

impl Handler for Observer {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = HaltReason;

    /// The default loop, recording each frame result before it goes back to its caller.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            self.record(&result);
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(init) => match evm.frame_init(init)? {
                    ItemOrResult::Item(_) => continue,
                    ItemOrResult::Result(result) => result,
                },
                ItemOrResult::Result(mut result) => {
                    evm.frame_run_result(&mut result)?;
                    result
                }
            };
            self.record(&result);
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }

    /// Records the settled transaction-level gas, then refunds as the default does.
    fn refund(
        &self,
        evm: &mut TestEvm,
        exec_result: &mut FrameResult,
        eip7702_refund: i64,
    ) -> Result<(), TestError> {
        self.settled.set(Some(*exec_result.gas().tracker()));
        post_execution::refund(
            evm.ctx().cfg().gas_params(),
            exec_result.gas_mut(),
            eip7702_refund,
        );
        Ok(())
    }
}

/// What the observed handler saw of one transaction.
struct Run {
    result: ExecutionResult,
    /// Every frame's gas as it returned, innermost first.
    returned: Vec<Returned>,
    /// The transaction-level gas once the first frame settled into it.
    settled: GasTracker,
}

impl Run {
    /// The transaction's own frame, as it returned.
    fn top(&self) -> Returned {
        *self.returned.last().expect("the transaction ran a frame")
    }

    /// The one frame [`CONTRACT`] started, as it returned.
    fn child(&self) -> Returned {
        assert_eq!(self.returned.len(), 2, "one child and the top frame");
        self.returned[0]
    }
}

/// How a run is configured: the switch, and whether the schedule sets the legacy shares.
#[derive(Clone, Copy, Debug)]
struct Setting {
    reservoir_only: bool,
    shares: bool,
}

const UPSTREAM: Setting = Setting {
    reservoir_only: false,
    shares: false,
};
const SATIN: Setting = Setting {
    reservoir_only: true,
    shares: true,
};
const RESERVOIR_ONLY: Setting = Setting {
    reservoir_only: true,
    shares: false,
};

fn context(db: Db, setting: Setting) -> TestContext {
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
        cfg.tx_gas_limit_cap = Some(REGULAR_CAP);
        cfg.reservoir_only_give_backs = setting.reservoir_only;
        if setting.shares {
            cfg.gas_params.override_gas([
                (GasId::sstore_set_state_gas_legacy_share(), SLOT_SHARE),
                (GasId::new_account_state_gas_legacy_share(), CALL_SHARE),
                (
                    GasId::selfdestruct_new_account_state_gas_legacy_share(),
                    SELFDESTRUCT_SHARE,
                ),
                (GasId::create_state_gas_legacy_share(), CREATE_SHARE),
            ]);
        }
    })
}

fn run(ctx: TestContext, tx: TxEnv) -> Run {
    let spec = ctx.cfg.spec;
    let instructions = EthInstructions::new_mainnet_with_spec(spec);
    let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(spec));
    evm.ctx.set_tx(tx);

    let mut observer = Observer::default();
    let result = observer.run(&mut evm).expect("transaction runs");
    Run {
        result,
        returned: observer.returned,
        settled: observer.settled.get().expect("post-execution ran"),
    }
}

/// A funded sender, [`CONTRACT`] holding `contract` and a balance of `balance`, and [`CHILD`]
/// holding `child`.
fn db(contract: Code, balance: u64, child: Code) -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code, balance) in [(CONTRACT, contract, balance), (CHILD, child, 0)] {
        let code = code.build();
        db.insert_account_info(
            address,
            AccountInfo::new(U256::from(balance), 1, code.hash_slow(), code),
        );
    }
    db
}

fn tx(kind: TxKind, data: Vec<u8>, gas_limit: u64) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(kind)
        .data(Bytes::from(data))
        .gas_price(0)
        .gas_limit(gas_limit)
        .build_fill()
}

/// Runs a call to [`CONTRACT`] with `reservoir` above the regular budget.
fn call(setting: Setting, contract: Code, balance: u64, child: Code, reservoir: u64) -> Run {
    run(
        context(db(contract, balance, child), setting),
        tx(TxKind::Call(CONTRACT), Vec::new(), REGULAR_CAP + reservoir),
    )
}

/// `(regular gas left, reservoir)` of a tracker.
const fn pools(gas: &GasTracker) -> (u64, u64) {
    (gas.remaining(), gas.reservoir())
}

/// The handler gives the transaction's gas and every frame's the switch the configuration sets.
#[test]
fn test_every_frames_gas_carries_the_configured_switch() {
    for setting in [UPSTREAM, RESERVOIR_ONLY] {
        let run = call(
            setting,
            Code::default().call(Some(100_000), CHILD, 0).op(STOP),
            0,
            Code::default().op(STOP),
            RESERVOIR,
        );
        assert_eq!(run.returned.len(), 2);
        for returned in &run.returned {
            assert_eq!(
                returned.gas.reservoir_only_give_backs(),
                setting.reservoir_only,
                "{setting:?}"
            );
        }
        assert_eq!(
            run.settled.reservoir_only_give_backs(),
            setting.reservoir_only
        );
    }
}

/// A fresh slot's legacy share comes from regular gas before the reservoir pays the rest: the
/// frame ends with the share less regular gas and the share more reservoir, the charge and the
/// total the same.
#[test]
fn test_a_fresh_slot_takes_its_legacy_share_from_regular_gas_first() {
    let code = || Code::default().sstore(1).op(STOP);
    let plain = call(RESERVOIR_ONLY, code(), 0, Code::default(), RESERVOIR);
    let shared = call(SATIN, code(), 0, Code::default(), RESERVOIR);

    let (regular, reservoir) = pools(&plain.top().gas);
    assert_eq!(reservoir, RESERVOIR - SSTORE_SET);
    assert_eq!(
        pools(&shared.top().gas),
        (regular - SLOT_SHARE, reservoir + SLOT_SHARE)
    );
    assert_eq!(shared.top().gas.state_gas_spent(), SSTORE_SET as i64);
    assert_eq!(
        shared.result.gas().total_gas_spent(),
        plain.result.gas().total_gas_spent()
    );
}

/// A frame whose regular gas cannot pay a fresh slot's legacy share runs out of gas, whatever
/// its reservoir holds; one regular gas more pays it.
#[test]
fn test_a_legacy_share_regular_gas_cannot_pay_is_an_out_of_gas() {
    let child = || Code::default().sstore(1).op(STOP);
    // What the child spends of its regular gas without a share.
    let measured = call(
        RESERVOIR_ONLY,
        Code::default().call(Some(100_000), CHILD, 0).op(STOP),
        0,
        child(),
        RESERVOIR,
    );
    let returned = measured.child();
    assert_eq!(returned.result, InstructionResult::Stop);
    let regular = returned.gas.limit() - returned.gas.remaining();

    for (forward, result) in [
        (regular + SLOT_SHARE - 1, InstructionResult::OutOfGas),
        (regular + SLOT_SHARE, InstructionResult::Stop),
    ] {
        let run = call(
            SATIN,
            Code::default().call(Some(forward), CHILD, 0).op(STOP),
            0,
            child(),
            RESERVOIR,
        );
        assert_eq!(run.child().result, result, "forward {forward}");
    }
}

/// The share of a value call's new account is taken before the forward is computed, so the
/// child is forwarded 63/64 of what the share left.
#[test]
fn test_a_value_calls_new_account_share_is_taken_before_the_forward() {
    let contract = || Code::default().call(None, EMPTY, 1).op(STOP);
    let plain = call(RESERVOIR_ONLY, contract(), 1, Code::default(), RESERVOIR);
    let shared = call(SATIN, contract(), 1, Code::default(), RESERVOIR);

    let forward = |remaining: u64| remaining - remaining / 64;
    let plain_limit = plain.child().gas.limit();
    let shared_limit = shared.child().gas.limit();
    let stipend = 2_300;
    // Invert the plain forward to the regular gas the caller held before it.
    let held = (0..64)
        .map(|r| (plain_limit - stipend) * 64 / 63 + r)
        .find(|held| forward(*held) == plain_limit - stipend)
        .expect("the forward inverts");
    assert_eq!(shared_limit - stipend, forward(held - CALL_SHARE));
    assert_eq!(
        shared.top().gas.reservoir() - plain.top().gas.reservoir(),
        CALL_SHARE,
        "regular gas paid the share, and the reservoir the rest"
    );
}

/// The share of a creation's account is taken before the forward is computed.
#[test]
fn test_a_creations_account_share_is_taken_before_the_forward() {
    let contract = || Code::default().create_and_stop();
    let plain = call(RESERVOIR_ONLY, contract(), 0, Code::default(), RESERVOIR);
    let shared = call(SATIN, contract(), 0, Code::default(), RESERVOIR);
    let forward = |remaining: u64| remaining - remaining / 64;

    let plain_limit = plain.child().gas.limit();
    let held = (0..64)
        .map(|r| plain_limit * 64 / 63 + r)
        .find(|held| forward(*held) == plain_limit)
        .expect("the forward inverts");
    assert_eq!(shared.child().gas.limit(), forward(held - CREATE_SHARE));
}

/// A `SELFDESTRUCT` that creates its beneficiary takes that account's share from regular gas
/// before the reservoir.
#[test]
fn test_a_selfdestructs_new_beneficiary_takes_its_share_from_regular_gas_first() {
    let contract = || Code::default().push_address(EMPTY).op(SELFDESTRUCT);
    let plain = call(RESERVOIR_ONLY, contract(), 1, Code::default(), RESERVOIR);
    let shared = call(SATIN, contract(), 1, Code::default(), RESERVOIR);

    let (regular, reservoir) = pools(&plain.top().gas);
    assert_eq!(reservoir, RESERVOIR - NEW_ACCOUNT);
    assert_eq!(
        pools(&shared.top().gas),
        (regular - SELFDESTRUCT_SHARE, reservoir + SELFDESTRUCT_SHARE)
    );
}

/// A slot written and written back gives its state gas back into the reservoir with the switch
/// on, though regular gas paid it, and to regular gas with the switch off.
#[test]
fn test_a_slot_written_back_gives_its_state_gas_to_the_reservoir() {
    let code = || Code::default().sstore(1).sstore(0).op(STOP);
    let upstream = call(UPSTREAM, code(), 0, Code::default(), 0);
    let reservoir_only = call(RESERVOIR_ONLY, code(), 0, Code::default(), 0);

    let (regular, reservoir) = pools(&upstream.top().gas);
    assert_eq!(reservoir, 0);
    assert_eq!(
        pools(&reservoir_only.top().gas),
        (regular - SSTORE_SET, SSTORE_SET)
    );
    assert_eq!(
        reservoir_only.top().gas.state_gas_spilled(),
        SSTORE_SET,
        "what regular gas paid stays counted"
    );
    assert_eq!(
        reservoir_only.result.gas().total_gas_spent(),
        upstream.result.gas().total_gas_spent(),
        "the reservoir returns to the sender with the regular gas"
    );
}

/// A child that pays a fresh slot from regular gas and reverts returns that charge in its
/// reservoir with the switch on, and as regular gas with it off: its caller holds the charge's
/// less regular gas and the charge more reservoir.
#[test]
fn test_a_reverting_child_returns_its_regular_paid_charge_as_reservoir() {
    let contract = || Code::default().call(Some(500_000), CHILD, 0).op(STOP);
    let child = || Code::default().sstore(1).revert();
    let upstream = call(UPSTREAM, contract(), 0, child(), 0);
    let reservoir_only = call(RESERVOIR_ONLY, contract(), 0, child(), 0);

    assert_eq!(upstream.child().result, InstructionResult::Revert);
    let (regular, reservoir) = pools(&upstream.top().gas);
    assert_eq!(reservoir, 0);
    assert_eq!(
        pools(&reservoir_only.top().gas),
        (regular - SSTORE_SET, SSTORE_SET)
    );
    assert_eq!(
        reservoir_only.top().gas.state_gas_spilled(),
        SSTORE_SET,
        "the caller counts what the child's regular gas paid"
    );
}

/// A child that pays a fresh slot from regular gas and halts returns the reservoir it received,
/// with the switch on as with it off: what regular gas paid is consumed with its regular gas, and
/// its caller ends as it does without the switch.
#[test]
fn test_a_halting_child_returns_the_reservoir_it_received() {
    let contract = || Code::default().call(Some(500_000), CHILD, 0).op(STOP);
    let child = || Code::default().sstore(1).op(INVALID);
    let upstream = call(UPSTREAM, contract(), 0, child(), 0);
    let reservoir_only = call(RESERVOIR_ONLY, contract(), 0, child(), 0);

    assert_eq!(upstream.child().result, InstructionResult::InvalidFEOpcode);
    assert_eq!(upstream.top().gas.reservoir(), 0);
    assert_eq!(pools(&reservoir_only.top().gas), pools(&upstream.top().gas));
    assert_eq!(reservoir_only.top().gas.state_gas_spilled(), 0);
}

/// A creation transaction whose frame halts gives back, with the switch on, the part of its
/// created account's state gas the reservoir paid, and consumes the part regular gas paid, as the
/// tracker without the switch does.
#[test]
fn test_a_halting_first_frame_gives_back_only_what_the_reservoir_paid_of_its_charge() {
    let reservoir = NEW_ACCOUNT / 3;
    for setting in [UPSTREAM, RESERVOIR_ONLY] {
        let run = run(
            context(db(Code::default(), 0, Code::default()), setting),
            tx(TxKind::Create, vec![INVALID], REGULAR_CAP + reservoir),
        );
        assert_eq!(run.top().result, InstructionResult::InvalidFEOpcode);
        assert_eq!(pools(&run.settled), (0, reservoir), "{setting:?}");
        assert!(matches!(run.result, ExecutionResult::Halt { .. }));
    }
}
