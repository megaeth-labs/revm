//! What revm's own EVM, which uses neither loop hook, gives an inspector and a caller: every
//! callback with what it is handed, and the transaction's result on the plain and the inspected
//! path. [`EXPECTED`] was recorded on the loops before they had the hooks (a pending action
//! returned before the first step; a frame's run result handed to the EVM before `frame_end`), so
//! the hooks' defaults change nothing a consumer that does not use them sees.
//!
//! [`PARENT`] calls a child that writes, logs and reverts, a precompile, and a contract that
//! self-destructs; creates a contract; logs; and returns. Each frame ends a different way.

use context::{result::ExecutionResult, Context, TxEnv};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{ExecuteEvm, FrameResult, MainBuilder, MainContext, MainnetContext, MainnetEvm};
use interpreter::{
    interpreter_types::Jumps, CallInputs, CallOutcome, CreateInputs, CreateOutcome, FrameInput,
    Interpreter,
};
use primitives::{address, hardfork::SpecId, Address, Log, TxKind, U256};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{
        OpCode, CALL, CREATE, INVALID, LOG0, LOG1, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3,
        RETURN, REVERT, SELFDESTRUCT, SSTORE,
    },
    AccountInfo, Bytecode, EvmState,
};

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 1_000_000;

/// Calls [`CHILD`], [`IDENTITY`] and [`DESTRUCT`], creates a contract, logs and returns.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Writes its slot 0, logs, and reverts with no output.
const CHILD: Address = address!("0x00000000000000000000000000000000000c0de1");
/// Self-destructs to the transaction's caller.
const DESTRUCT: Address = address!("0x00000000000000000000000000000000000c0de2");
/// The identity precompile.
const IDENTITY: Address = address!("0x0000000000000000000000000000000000000004");
/// The gas [`PARENT`] forwards to each call: 0x0186a0.
const CALL_GAS: [u8; 3] = [0x01, 0x86, 0xa0];
/// The creation's init code, which returns one `INVALID` byte as the contract's code.
const INIT_CODE: [u8; 9] = [PUSH1, INVALID, PUSH0, MSTORE, PUSH1, 1, PUSH1, 31, RETURN];

/// Every callback, one line each, with what it is handed.
#[derive(Debug, Default)]
struct Recorder {
    events: Vec<String>,
}

fn name(address: Address) -> String {
    match address {
        PARENT => "PARENT".into(),
        CHILD => "CHILD".into(),
        DESTRUCT => "DESTRUCT".into(),
        IDENTITY => "IDENTITY".into(),
        BENCH_CALLER => "CALLER".into(),
        other => other.to_string(),
    }
}

fn frame(input: &FrameInput) -> String {
    match input {
        FrameInput::Call(inputs) => format!("call {}", name(inputs.target_address)),
        FrameInput::Create(inputs) => format!("create by {}", name(inputs.caller())),
        FrameInput::Empty => "empty".into(),
    }
}

fn ended(result: &FrameResult) -> String {
    let (interpreter_result, gas) = (result.interpreter_result(), result.gas());
    format!(
        "{:?} {} remaining={} refunded={} reservoir={}",
        interpreter_result.result,
        interpreter_result.output,
        gas.remaining(),
        gas.refunded(),
        gas.reservoir()
    )
}

fn at(interp: &Interpreter) -> String {
    let pc = interp.bytecode.pc();
    let opcode = OpCode::new(interp.bytecode.opcode()).map_or("?", OpCode::as_str);
    format!(
        "{} {pc} {opcode} gas={} stack={}",
        name(interp.input.target_address),
        interp.gas.remaining(),
        interp.stack.len()
    )
}

impl Inspector<Ctx> for Recorder {
    fn initialize_interp(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        self.events
            .push(format!("initialize_interp {}", at(interp)));
    }

    fn step(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        self.events.push(format!("step {}", at(interp)));
    }

    fn step_end(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        self.events.push(format!("step_end {}", at(interp)));
    }

    fn log(&mut self, _: &mut Ctx, log: Log) {
        self.events.push(format!("log {log:?}"));
    }

    fn log_full(&mut self, interp: &mut Interpreter, _: &mut Ctx, log: Log) {
        self.events.push(format!("log_full {} {log:?}", at(interp)));
    }

    fn frame_start(&mut self, _: &mut Ctx, input: &mut FrameInput) -> Option<FrameResult> {
        self.events.push(format!("frame_start {}", frame(input)));
        None
    }

    fn frame_end(&mut self, _: &mut Ctx, input: &FrameInput, result: &mut FrameResult) {
        let line = format!("frame_end {} {}", frame(input), ended(result));
        self.events.push(line);
    }

    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let (target, gas) = (name(inputs.target_address), inputs.gas_limit);
        self.events.push(format!("call {target} gas={gas}"));
        None
    }

    fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let result = ended(&FrameResult::Call(outcome.clone()));
        let target = name(inputs.target_address);
        self.events.push(format!("call_end {target} {result}"));
    }

    fn create(&mut self, _: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let (caller, gas) = (name(inputs.caller()), inputs.gas_limit());
        self.events.push(format!("create by {caller} gas={gas}"));
        None
    }

    fn create_end(&mut self, _: &mut Ctx, _: &CreateInputs, outcome: &mut CreateOutcome) {
        let address = outcome.address.map(name);
        let result = ended(&FrameResult::Create(outcome.clone()));
        self.events.push(format!("create_end {address:?} {result}"));
    }

    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let (contract, target) = (name(contract), name(target));
        self.events
            .push(format!("selfdestruct {contract} to {target} value={value}"));
    }
}

/// Appends a call to `target` with [`CALL_GAS`] and no input, whose status it pops.
fn call(code: &mut Vec<u8>, target: Address) {
    code.extend_from_slice(&[PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20]);
    code.extend_from_slice(target.as_slice());
    code.push(PUSH3);
    code.extend_from_slice(&CALL_GAS);
    code.extend_from_slice(&[CALL, POP]);
}

fn parent_code() -> Bytecode {
    let mut code = Vec::new();
    call(&mut code, CHILD);
    call(&mut code, IDENTITY);
    call(&mut code, DESTRUCT);
    // The init code at the end of the first word, then CREATE(0, 32 - len, len).
    code.push(PUSH1 + INIT_CODE.len() as u8 - 1);
    code.extend_from_slice(&INIT_CODE);
    code.extend_from_slice(&[PUSH0, MSTORE]);
    code.extend_from_slice(&[
        PUSH1,
        INIT_CODE.len() as u8,
        PUSH1,
        32 - INIT_CODE.len() as u8,
        PUSH0,
        CREATE,
        POP,
    ]);
    code.extend_from_slice(&[PUSH1, 7, PUSH0, PUSH0, LOG1]);
    code.extend_from_slice(&[PUSH1, 32, PUSH0, RETURN]);
    Bytecode::new_raw(code.into())
}

fn child_code() -> Bytecode {
    Bytecode::new_raw(
        vec![
            PUSH1, 1, PUSH0, SSTORE, PUSH0, PUSH0, LOG0, PUSH0, PUSH0, REVERT,
        ]
        .into(),
    )
}

fn destruct_code() -> Bytecode {
    let mut code = vec![PUSH20];
    code.extend_from_slice(BENCH_CALLER.as_slice());
    code.push(SELFDESTRUCT);
    Bytecode::new_raw(code.into())
}

fn evm() -> MainnetEvm<Ctx, Recorder> {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code, balance) in [
        (PARENT, parent_code(), 0),
        (CHILD, child_code(), 0),
        (DESTRUCT, destruct_code(), 5),
    ] {
        let info = AccountInfo::new(U256::from(balance), 1, code.hash_slow(), code);
        db.insert_account_info(address, info);
    }
    Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SPEC))
        .build_mainnet_with_inspector(Recorder::default())
}

fn tx() -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(PARENT))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill()
}

/// The result, and the state the transaction left, one line per account.
fn summary(result: &ExecutionResult, state: EvmState) -> Vec<String> {
    let mut lines = vec![format!("result {result:?}")];
    let mut accounts: Vec<_> = state.into_iter().collect();
    accounts.sort_by_key(|(address, _)| *address);
    for (address, account) in accounts {
        let mut slots: Vec<_> = account
            .storage
            .iter()
            .map(|(slot, value)| (*slot, value.present_value))
            .collect();
        slots.sort();
        lines.push(format!(
            "account {} balance={} nonce={} code={} slots={slots:?}",
            name(address),
            account.info.balance,
            account.info.nonce,
            account.info.code_hash
        ));
    }
    lines
}

/// The inspector's lines, then the result and state lines, as recorded on the loops without the
/// hooks.
const EXPECTED: &[&str] = &[
    "frame_start call PARENT",
    "call PARENT gas=979000",
    "initialize_interp PARENT 0 PUSH0 gas=979000 stack=0",
    "step PARENT 0 PUSH0 gas=979000 stack=0",
    "step_end PARENT 1 PUSH0 gas=978998 stack=1",
    "step PARENT 1 PUSH0 gas=978998 stack=1",
    "step_end PARENT 2 PUSH0 gas=978996 stack=2",
    "step PARENT 2 PUSH0 gas=978996 stack=2",
    "step_end PARENT 3 PUSH0 gas=978994 stack=3",
    "step PARENT 3 PUSH0 gas=978994 stack=3",
    "step_end PARENT 4 PUSH0 gas=978992 stack=4",
    "step PARENT 4 PUSH0 gas=978992 stack=4",
    "step_end PARENT 5 PUSH20 gas=978990 stack=5",
    "step PARENT 5 PUSH20 gas=978990 stack=5",
    "step_end PARENT 26 PUSH3 gas=978987 stack=6",
    "step PARENT 26 PUSH3 gas=978987 stack=6",
    "step_end PARENT 30 CALL gas=978984 stack=7",
    "step PARENT 30 CALL gas=978984 stack=7",
    "step_end PARENT 31 POP gas=876384 stack=0",
    "frame_start call CHILD",
    "call CHILD gas=100000",
    "initialize_interp CHILD 0 PUSH1 gas=100000 stack=0",
    "step CHILD 0 PUSH1 gas=100000 stack=0",
    "step_end CHILD 2 PUSH0 gas=99997 stack=1",
    "step CHILD 2 PUSH0 gas=99997 stack=1",
    "step_end CHILD 3 SSTORE gas=99995 stack=2",
    "step CHILD 3 SSTORE gas=99995 stack=2",
    "step_end CHILD 4 PUSH0 gas=77895 stack=0",
    "step CHILD 4 PUSH0 gas=77895 stack=0",
    "step_end CHILD 5 PUSH0 gas=77893 stack=1",
    "step CHILD 5 PUSH0 gas=77893 stack=1",
    "step_end CHILD 6 LOG0 gas=77891 stack=2",
    "step CHILD 6 LOG0 gas=77891 stack=2",
    "log_full CHILD 7 PUSH0 gas=77516 stack=0 Log { address: 0x00000000000000000000000000000000000c0de1, data: LogData { topics: [], data: 0x } }",
    "step_end CHILD 7 PUSH0 gas=77516 stack=0",
    "step CHILD 7 PUSH0 gas=77516 stack=0",
    "step_end CHILD 8 PUSH0 gas=77514 stack=1",
    "step CHILD 8 PUSH0 gas=77514 stack=1",
    "step_end CHILD 9 REVERT gas=77512 stack=2",
    "step CHILD 9 REVERT gas=77512 stack=2",
    "step_end CHILD 10 STOP gas=77512 stack=0",
    "call_end CHILD Revert 0x remaining=77512 refunded=0 reservoir=0",
    "frame_end call CHILD Revert 0x remaining=77512 refunded=0 reservoir=0",
    "step PARENT 31 POP gas=953896 stack=1",
    "step_end PARENT 32 PUSH0 gas=953894 stack=0",
    "step PARENT 32 PUSH0 gas=953894 stack=0",
    "step_end PARENT 33 PUSH0 gas=953892 stack=1",
    "step PARENT 33 PUSH0 gas=953892 stack=1",
    "step_end PARENT 34 PUSH0 gas=953890 stack=2",
    "step PARENT 34 PUSH0 gas=953890 stack=2",
    "step_end PARENT 35 PUSH0 gas=953888 stack=3",
    "step PARENT 35 PUSH0 gas=953888 stack=3",
    "step_end PARENT 36 PUSH0 gas=953886 stack=4",
    "step PARENT 36 PUSH0 gas=953886 stack=4",
    "step_end PARENT 37 PUSH20 gas=953884 stack=5",
    "step PARENT 37 PUSH20 gas=953884 stack=5",
    "step_end PARENT 58 PUSH3 gas=953881 stack=6",
    "step PARENT 58 PUSH3 gas=953881 stack=6",
    "step_end PARENT 62 CALL gas=953878 stack=7",
    "step PARENT 62 CALL gas=953878 stack=7",
    "step_end PARENT 63 POP gas=853778 stack=0",
    "frame_start call IDENTITY",
    "call IDENTITY gas=100000",
    "call_end IDENTITY Return 0x remaining=99985 refunded=0 reservoir=0",
    "frame_end call IDENTITY Return 0x remaining=99985 refunded=0 reservoir=0",
    "step PARENT 63 POP gas=953763 stack=1",
    "step_end PARENT 64 PUSH0 gas=953761 stack=0",
    "step PARENT 64 PUSH0 gas=953761 stack=0",
    "step_end PARENT 65 PUSH0 gas=953759 stack=1",
    "step PARENT 65 PUSH0 gas=953759 stack=1",
    "step_end PARENT 66 PUSH0 gas=953757 stack=2",
    "step PARENT 66 PUSH0 gas=953757 stack=2",
    "step_end PARENT 67 PUSH0 gas=953755 stack=3",
    "step PARENT 67 PUSH0 gas=953755 stack=3",
    "step_end PARENT 68 PUSH0 gas=953753 stack=4",
    "step PARENT 68 PUSH0 gas=953753 stack=4",
    "step_end PARENT 69 PUSH20 gas=953751 stack=5",
    "step PARENT 69 PUSH20 gas=953751 stack=5",
    "step_end PARENT 90 PUSH3 gas=953748 stack=6",
    "step PARENT 90 PUSH3 gas=953748 stack=6",
    "step_end PARENT 94 CALL gas=953745 stack=7",
    "step PARENT 94 CALL gas=953745 stack=7",
    "step_end PARENT 95 POP gas=851145 stack=0",
    "frame_start call DESTRUCT",
    "call DESTRUCT gas=100000",
    "initialize_interp DESTRUCT 0 PUSH20 gas=100000 stack=0",
    "step DESTRUCT 0 PUSH20 gas=100000 stack=0",
    "step_end DESTRUCT 21 SELFDESTRUCT gas=99997 stack=1",
    "step DESTRUCT 21 SELFDESTRUCT gas=99997 stack=1",
    "step_end DESTRUCT 22 STOP gas=94997 stack=0",
    "selfdestruct DESTRUCT to CALLER value=5",
    "call_end DESTRUCT SelfDestruct 0x remaining=94997 refunded=0 reservoir=0",
    "frame_end call DESTRUCT SelfDestruct 0x remaining=94997 refunded=0 reservoir=0",
    "step PARENT 95 POP gas=946142 stack=1",
    "step_end PARENT 96 PUSH9 gas=946140 stack=0",
    "step PARENT 96 PUSH9 gas=946140 stack=0",
    "step_end PARENT 106 PUSH0 gas=946137 stack=1",
    "step PARENT 106 PUSH0 gas=946137 stack=1",
    "step_end PARENT 107 MSTORE gas=946135 stack=2",
    "step PARENT 107 MSTORE gas=946135 stack=2",
    "step_end PARENT 108 PUSH1 gas=946129 stack=0",
    "step PARENT 108 PUSH1 gas=946129 stack=0",
    "step_end PARENT 110 PUSH1 gas=946126 stack=1",
    "step PARENT 110 PUSH1 gas=946126 stack=1",
    "step_end PARENT 112 PUSH0 gas=946123 stack=2",
    "step PARENT 112 PUSH0 gas=946123 stack=2",
    "step_end PARENT 113 CREATE gas=946121 stack=3",
    "step PARENT 113 CREATE gas=946121 stack=3",
    "step_end PARENT 114 POP gas=14283 stack=0",
    "frame_start create by PARENT",
    "create by PARENT gas=899836",
    "initialize_interp 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 0 PUSH1 gas=899836 stack=0",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 0 PUSH1 gas=899836 stack=0",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 2 PUSH0 gas=899833 stack=1",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 2 PUSH0 gas=899833 stack=1",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 3 MSTORE gas=899831 stack=2",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 3 MSTORE gas=899831 stack=2",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 4 PUSH1 gas=899825 stack=0",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 4 PUSH1 gas=899825 stack=0",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 6 PUSH1 gas=899822 stack=1",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 6 PUSH1 gas=899822 stack=1",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 8 RETURN gas=899819 stack=2",
    "step 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 8 RETURN gas=899819 stack=2",
    "step_end 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 9 STOP gas=899819 stack=0",
    "create_end Some(\"0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18\") Return 0xfe remaining=899619 refunded=0 reservoir=0",
    "frame_end create by PARENT Return 0xfe remaining=899619 refunded=0 reservoir=0",
    "step PARENT 114 POP gas=913902 stack=1",
    "step_end PARENT 115 PUSH1 gas=913900 stack=0",
    "step PARENT 115 PUSH1 gas=913900 stack=0",
    "step_end PARENT 117 PUSH0 gas=913897 stack=1",
    "step PARENT 117 PUSH0 gas=913897 stack=1",
    "step_end PARENT 118 PUSH0 gas=913895 stack=2",
    "step PARENT 118 PUSH0 gas=913895 stack=2",
    "step_end PARENT 119 LOG1 gas=913893 stack=3",
    "step PARENT 119 LOG1 gas=913893 stack=3",
    "log_full PARENT 120 PUSH1 gas=913143 stack=0 Log { address: 0x00000000000000000000000000000000000c0de0, data: LogData { topics: [0x0000000000000000000000000000000000000000000000000000000000000007], data: 0x } }",
    "step_end PARENT 120 PUSH1 gas=913143 stack=0",
    "step PARENT 120 PUSH1 gas=913143 stack=0",
    "step_end PARENT 122 PUSH0 gas=913140 stack=1",
    "step PARENT 122 PUSH0 gas=913140 stack=1",
    "step_end PARENT 123 RETURN gas=913138 stack=2",
    "step PARENT 123 RETURN gas=913138 stack=2",
    "step_end PARENT 124 STOP gas=913138 stack=0",
    "call_end PARENT Return 0x000000000000000000000000000000000000000000000060fe5f526001601ff3 remaining=913138 refunded=0 reservoir=0",
    "frame_end call PARENT Return 0x000000000000000000000000000000000000000000000060fe5f526001601ff3 remaining=913138 refunded=0 reservoir=0",
    "result Success { reason: Return, gas: ResultGas { total_gas_spent: 86862, state_gas_spent: 0, refunded: 0, floor_gas: 21000 }, logs: [Log { address: 0x00000000000000000000000000000000000c0de0, data: LogData { topics: [0x0000000000000000000000000000000000000000000000000000000000000007], data: 0x } }], output: Call(0x000000000000000000000000000000000000000000000060fe5f526001601ff3) }",
    "account 0x0000000000000000000000000000000000000000 balance=0 nonce=0 code=0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 slots=[]",
    "account IDENTITY balance=0 nonce=0 code=0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 slots=[]",
    "account PARENT balance=0 nonce=2 code=0xdbf003432a6e20a1c2e13f77f0130cb233125ef7784a3402e451f5fab5fb380d slots=[]",
    "account CHILD balance=0 nonce=1 code=0x67fda313dc72b1739f04ae407f6386f7db6466b9ee221a1c91ca0e0dbf9f648a slots=[(0, 0)]",
    "account DESTRUCT balance=0 nonce=1 code=0x7ff285cf123c2738db918e966277ec00263c11d3942a177731207a462269db36 slots=[]",
    "account 0xed591dC4375A9E8c959dA1AAa3712ac1063fCC18 balance=0 nonce=1 code=0xbcc90f2d6dada5b18e155c17a1c0a55920aae94f39857d39d0d8ed07ae8f228b slots=[]",
    "account CALLER balance=1000000000000000000005 nonce=1 code=0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 slots=[]",
];

/// The inspector's callbacks, and the result and state of both paths, are as recorded.
#[test]
fn test_an_evm_without_the_loop_hooks_gives_the_recorded_callbacks_and_results() {
    let mut inspected = evm();
    let result = inspected
        .inspect_one_tx(tx())
        .expect("the transaction runs");
    let state = inspected.finalize();
    let mut seen = core::mem::take(&mut inspected.inspector.events);
    seen.extend(summary(&result, state));

    let mut plain = evm();
    let result = plain.transact_one(tx()).expect("the transaction runs");
    let plain = summary(&result, plain.finalize());

    assert_eq!(seen[seen.len() - plain.len()..], plain, "the plain path");
    assert_eq!(seen, EXPECTED);
}
