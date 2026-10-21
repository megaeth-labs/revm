//! Serde encoding of the byte-pool fields on the frame inputs and outcomes: the child's share,
//! the start charge and a call's history allowance.
//!
//! New encodings carry the fields. In a format that carries field names, such as the JSON these
//! tests use, encodings made before they existed lack them and still decode: to an unlimited
//! share, no start charge and no allowance, which is what a frame of a consumer that uses none of
//! them carries. A format that encodes a struct by position, such as bincode, cannot read an
//! encoding made before they existed, as with upstream's own field additions.
#![cfg(feature = "serde")]

use core::fmt::Debug;
use primitives::{address, Bytes, U256};
use revm_interpreter::{
    CallInput, CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome,
    CreateScheme, Gas, InstructionResult, InterpreterResult, StartCharge,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use state::Bytecode;

const CHARGE: StartCharge = StartCharge::new(3_520, 160);

/// Checks that `value` round-trips, and decodes to `without` once `fields` are removed from its
/// encoding.
fn assert_decodes_with_and_without<T>(value: &T, fields: &[&str], without: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let encoded = serde_json::to_value(value).unwrap();
    assert_eq!(
        &serde_json::from_value::<T>(encoded.clone()).unwrap(),
        value
    );

    let Value::Object(mut object) = encoded else {
        panic!("expected a JSON object, got {encoded}");
    };
    for field in fields {
        assert!(object.remove(*field).is_some(), "{field} is encoded");
    }
    assert_eq!(
        &serde_json::from_value::<T>(Value::Object(object)).unwrap(),
        without
    );
}

const fn interpreter_result() -> InterpreterResult {
    InterpreterResult::new(
        InstructionResult::Revert,
        Bytes::from_static(b"reverted"),
        Gas::new(100_000),
    )
}

#[test]
fn test_call_inputs_decode_from_json_with_and_without_the_pool_fields() {
    let inputs = |share, charge, allowance| CallInputs {
        input: CallInput::Bytes(Bytes::from_static(b"in")),
        return_memory_offset: 0..8,
        gas_limit: 100_000,
        reservoir: 50_000,
        bytecode_address: address!("0x00000000000000000000000000000000000c0de0"),
        known_bytecode: (Default::default(), Bytecode::default()),
        target_address: address!("0x00000000000000000000000000000000000c0de0"),
        caller: address!("0x00000000000000000000000000000000000ca11e"),
        value: CallValue::Transfer(U256::from(1)),
        scheme: CallScheme::Call,
        is_static: false,
        charged_new_account_state_gas: true,
        charged_start: charge,
        bytes_share: share,
        history_allowance: allowance,
    };

    assert_decodes_with_and_without(
        &inputs(980, CHARGE, 160),
        &["bytes_share", "charged_start", "history_allowance"],
        &inputs(u64::MAX, StartCharge::default(), 0),
    );
}

#[test]
fn test_create_inputs_decode_from_json_with_and_without_the_pool_fields() {
    let inputs = |share, charge| {
        let mut inputs = CreateInputs::new(
            address!("0x00000000000000000000000000000000000c0de0"),
            CreateScheme::Create,
            U256::from(1),
            Bytes::from_static(&[0x5f, 0x5f, 0xfd]),
            100_000,
            50_000,
        );
        inputs.set_bytes_share(share);
        inputs.set_charged_start(charge);
        inputs
    };

    assert_decodes_with_and_without(
        &inputs(980, CHARGE),
        &["bytes_share", "charged_start"],
        &inputs(u64::MAX, StartCharge::default()),
    );
    assert_eq!(CreateInputs::default().bytes_share(), u64::MAX);
}

#[test]
fn test_call_outcome_decodes_from_json_with_and_without_the_start_charge() {
    let outcome = |charge| {
        let mut outcome = CallOutcome::new(interpreter_result(), 0..8);
        outcome.charged_start = charge;
        outcome
    };

    assert_decodes_with_and_without(
        &outcome(CHARGE),
        &["charged_start"],
        &outcome(StartCharge::default()),
    );
}

#[test]
fn test_create_outcome_decodes_from_json_with_and_without_the_start_charge() {
    let outcome = |charge| {
        let mut outcome = CreateOutcome::new(interpreter_result(), None);
        outcome.charged_start = charge;
        outcome
    };

    assert_decodes_with_and_without(
        &outcome(CHARGE),
        &["charged_start"],
        &outcome(StartCharge::default()),
    );
}
