//! Serde encoding of `charged_state_gas_address` on the frame inputs and outcomes.
//!
//! New encodings carry the field. Encodings made before it existed lack it and still decode, to
//! [`Address::ZERO`], which is what a frame without a charge carries.
#![cfg(feature = "serde")]

use core::fmt::Debug;
use primitives::{address, Address, Bytes, U256};
use revm_interpreter::{
    CallOutcome, CreateInputs, CreateOutcome, CreateScheme, Gas, InstructionResult,
    InterpreterResult,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

const FIELD: &str = "charged_state_gas_address";

/// The account a charge was priced for.
const CHARGED: Address = address!("0x00000000000000000000000000000000000c4a26");

/// Checks that `value` round-trips with the field and decodes to `without_field` once the field
/// is removed from its encoding.
fn assert_decodes_with_and_without_field<T>(value: &T, without_field: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let encoded = serde_json::to_value(value).unwrap();
    assert_eq!(
        &serde_json::from_value::<T>(encoded.clone()).unwrap(),
        value
    );

    let Value::Object(mut fields) = encoded else {
        panic!("expected a JSON object, got {encoded}");
    };
    assert_eq!(
        fields.remove(FIELD),
        Some(Value::String(CHARGED.to_string().to_lowercase())),
    );
    let old = Value::Object(fields);
    assert_eq!(&serde_json::from_value::<T>(old).unwrap(), without_field);
}

fn interpreter_result() -> InterpreterResult {
    InterpreterResult::new(
        InstructionResult::Revert,
        Bytes::from_static(b"reverted"),
        Gas::new(100_000),
    )
}

#[test]
fn test_create_inputs_decode_with_and_without_the_charged_address() {
    let inputs = |charged_address| {
        let mut inputs = CreateInputs::new(
            address!("0x00000000000000000000000000000000000c0de0"),
            CreateScheme::Create2 {
                salt: U256::from(7),
            },
            U256::from(1),
            Bytes::from_static(&[0x5f, 0x5f, 0xfd]),
            100_000,
            50_000,
        );
        inputs.set_charged_create_state_gas(true);
        inputs.set_charged_state_gas_address(charged_address);
        inputs
    };

    assert_decodes_with_and_without_field(&inputs(CHARGED), &inputs(Address::ZERO));
}

#[test]
fn test_call_outcome_decodes_with_and_without_the_charged_address() {
    let outcome = |charged_address| {
        let mut outcome = CallOutcome::new(interpreter_result(), 0..8);
        outcome.charged_new_account_state_gas = true;
        outcome.charged_state_gas_address = charged_address;
        outcome
    };

    assert_decodes_with_and_without_field(&outcome(CHARGED), &outcome(Address::ZERO));
}

#[test]
fn test_create_outcome_decodes_with_and_without_the_charged_address() {
    let outcome = |charged_address| {
        let created = address!("0x000000000000000000000000000000000000c4ea");
        let mut outcome = CreateOutcome::new(interpreter_result(), Some(created));
        outcome.charged_create_state_gas = true;
        outcome.charged_state_gas_address = charged_address;
        outcome
    };

    assert_decodes_with_and_without_field(&outcome(CHARGED), &outcome(Address::ZERO));
}
