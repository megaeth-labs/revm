//! Serialized form of the system call state-gas margin switch
//! ([`CfgEnv::system_call_state_gas_margin_in_reservoir`]).
//!
//! A payload written before the switch existed has no key for it and must decode with the switch
//! off.
#![cfg(feature = "serde")]

use primitives::hardfork::SpecId;
use revm_context::CfgEnv;

#[test]
fn test_cfg_env_json_without_system_call_margin_switch_decodes_to_false() {
    let cfg = CfgEnv::new_with_spec(SpecId::AMSTERDAM)
        .with_system_call_state_gas_margin_in_reservoir(true);
    let mut value = serde_json::to_value(&cfg).expect("CfgEnv serializes");
    let object = value.as_object_mut().expect("CfgEnv JSON is an object");
    assert_eq!(
        object.remove("system_call_state_gas_margin_in_reservoir"),
        Some(serde_json::Value::Bool(true))
    );

    let decoded: CfgEnv = serde_json::from_value(value).expect("missing field uses serde default");
    assert!(!decoded.system_call_state_gas_margin_in_reservoir);
    assert_eq!(
        decoded,
        cfg.with_system_call_state_gas_margin_in_reservoir(false)
    );
}

#[test]
fn test_cfg_env_system_call_margin_switch_true_roundtrips() {
    let cfg = CfgEnv::new_with_spec(SpecId::AMSTERDAM)
        .with_system_call_state_gas_margin_in_reservoir(true);
    let json = serde_json::to_string(&cfg).expect("CfgEnv serializes");
    let decoded: CfgEnv = serde_json::from_str(&json).expect("CfgEnv deserializes");
    assert!(decoded.system_call_state_gas_margin_in_reservoir);
    assert_eq!(decoded, cfg);
}
