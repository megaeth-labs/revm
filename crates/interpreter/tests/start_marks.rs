//! The marks revm writes on a frame result before an inspector's end callbacks keep the values'
//! public semantics: a marked gas tracker and a marked outcome print exactly as the derived debug
//! output printed them before the marks existed, and encode without them.

use revm_interpreter::{
    CallOutcome, CreateOutcome, Gas, GasTracker, InstructionResult, InterpreterResult, OutcomeMark,
    StartMark,
};

/// The derived debug output of these values before the marks existed.
const TRACKER: &str = "GasTracker { gas_limit: 100, remaining: 90, reservoir: 5, \
    state_gas_spent: 0, state_gas_spilled: 0, history_gas_spent: 0, refunded: 3, withheld: 0, \
    withheld_crossing: None, failed_charge_remaining: 0, failed_charge_recorded: false }";
const RESULT: &str = "InterpreterResult { result: Stop, output: 0x, gas: Gas { tracker: \
    GasTracker { gas_limit: 10, remaining: 10, reservoir: 0, state_gas_spent: 0, \
    state_gas_spilled: 0, history_gas_spent: 0, refunded: 0, withheld: 0, withheld_crossing: \
    None, failed_charge_remaining: 0, failed_charge_recorded: false }, memory: MemoryGas { \
    words_num: 0, expansion_cost: 0 } } }";

const fn tracker() -> GasTracker {
    let mut tracker = GasTracker::new(100, 90, 5);
    tracker.record_refund(3);
    tracker.set_start_mark(StartMark::for_start(7));
    tracker
}

fn result() -> InterpreterResult {
    let mut gas = Gas::new(10);
    gas.tracker_mut().set_start_mark(StartMark::for_start(7));
    InterpreterResult::new(InstructionResult::Stop, Default::default(), gas)
}

fn call() -> CallOutcome {
    CallOutcome {
        start_mark: OutcomeMark::for_start(7),
        ..CallOutcome::new(result(), 1..2)
    }
}

fn create() -> CreateOutcome {
    CreateOutcome {
        start_mark: OutcomeMark::for_start(7),
        ..CreateOutcome::new(result(), None)
    }
}

#[test]
fn test_marked_values_print_as_the_derived_output_did() {
    assert_eq!(format!("{:?}", tracker()), TRACKER);
    assert_eq!(
        format!("{:?}", call()),
        format!(
            "CallOutcome {{ result: {RESULT}, memory_offset: 1..2, was_precompile_called: false, \
             precompile_call_logs: [], charged_new_account_state_gas: false, \
             charged_state_gas_address: 0x0000000000000000000000000000000000000000 }}"
        )
    );
    assert_eq!(
        format!("{:?}", create()),
        format!(
            "CreateOutcome {{ result: {RESULT}, address: None, charged_create_state_gas: false, \
             charged_state_gas_address: 0x0000000000000000000000000000000000000000 }}"
        )
    );
}

#[test]
fn test_marked_values_equal_their_unmarked_selves() {
    let mut unmarked = tracker();
    unmarked.set_start_mark(StartMark::NONE);
    assert_eq!(tracker(), unmarked);
    assert_eq!(call(), CallOutcome::new(result(), 1..2));
    assert_eq!(create(), CreateOutcome::new(result(), None));
}

/// An encoded value has no mark, and decodes equal with none.
#[cfg(feature = "serde")]
#[test]
fn test_marked_values_encode_without_their_marks() {
    let json = serde_json::to_string(&tracker()).expect("encodes");
    assert!(!json.contains("mark"), "{json}");
    let decoded: GasTracker = serde_json::from_str(&json).expect("decodes");
    assert_eq!(decoded, tracker());
    assert!(!decoded.start_mark().is_for(7));

    let json = serde_json::to_string(&call()).expect("encodes");
    assert!(!json.contains("mark"), "{json}");
    let decoded: CallOutcome = serde_json::from_str(&json).expect("decodes");
    assert_eq!(decoded, call());
    assert!(!decoded.start_mark.is_for(7));
    assert!(!decoded.result.gas.tracker().start_mark().is_for(7));

    let json = serde_json::to_string(&create()).expect("encodes");
    assert!(!json.contains("mark"), "{json}");
    let decoded: CreateOutcome = serde_json::from_str(&json).expect("decodes");
    assert_eq!(decoded, create());
    assert!(!decoded.start_mark.is_for(7));
}
