//! The gas a precompile call needs, known before the precompile runs.
//!
//! [`Precompile::required_gas`] answers, for an input, the gas limit below which running the
//! precompile halts out of gas, without running it. A consumer that must decide whether a call is
//! affordable before the precompile does any work, rather than read the answer off the run,
//! asks here.
//!
//! # Contract
//!
//! `required_gas(input) == Some(price)` guarantees, for every gas limit `g`:
//!
//! - [`Precompile::execute`] with `g` halts with
//!   [`PrecompileHalt::OutOfGas`](crate::PrecompileHalt::OutOfGas) if and only if
//!   `g < price`;
//! - with `g >= price` the run's result is the same for every `g`, and a run that succeeds or
//!   reverts uses exactly `price`.
//!
//! So `price` is both what the call charges and the least gas limit it runs on. An input that
//! fails a check, a malformed one, keeps the contract too:
//!
//! - a check made before any gas check fails the call the same way at every gas limit, so the
//!   input's price is `0`: it needs no gas to fail. The BLS12-381 MSM and pairing precompiles
//!   refuse a length that is not a positive multiple of their element this way, and `BLAKE2F`
//!   one that is not 213 bytes;
//! - a check made after a gas check fails the call only once that gas check passed, so the
//!   input's price is the gas that check asks for: every precompile with a flat price, the
//!   BN254 pairing on a length that is not a multiple of 192 bytes, KZG point evaluation on any
//!   malformed input, a `BLAKE2F` final-block flag other than 0 or 1;
//! - `MODEXP` checks the sizes in its header between its two gas checks: a header whose sizes
//!   do not fit (EIP-7823 under Osaka, or `usize` everywhere) prices at the minimum the first
//!   gas check asks for, 0, 200 or 500 by schedule.
//!
//! The price never tells whether a call with enough gas succeeds: an input that fails a check
//! after the gas check is only told apart from one that succeeds by running it.
//!
//! # Which precompiles answer
//!
//! Every precompile this crate defines carries its price function, so every precompile of every
//! set [`Precompiles::new`](crate::Precompiles::new) returns answers. A precompile built with
//! [`Precompile::new`] carries none and answers `None`, so a precompile defined elsewhere keeps
//! compiling and behaving as before; it opts in with [`Precompile::with_required_gas`]. The price
//! functions are public, so a precompile that wraps one of this crate's, such as one that adds an
//! input size limit in front of it, can price itself from the one it wraps; it must, and must
//! price its own refusals at `0` (see [`Precompile::with_required_gas`]).
//!
//! # Price what is dispatched
//!
//! The price is a property of the [`Precompile`] value, and holds only for that value's run. A
//! consumer takes it from the value it dispatches for the address, the one its EVM will run:
//!
//! - a table that type-erases its entries cannot be priced through. alloy-evm's
//!   `PrecompilesMap` is one once it holds dynamic precompiles: its entries are no longer
//!   [`Precompile`] values and carry no price;
//! - an address a node replaced with a precompile of its own must not be priced from the
//!   built-in set. The built-in value still answers for that address, with the price of a run
//!   that no longer happens.
//!
//! Nothing here can see what a consumer dispatches, so this is the consumer's to keep.
//!
//! # A call held to less than its forward
//!
//! A consumer that holds a call to an `allowance` below the `forward` its caller gave it tells
//! from the price, without running the precompile, which of three cases the call is in:
//!
//! - `price <= allowance`: the call is affordable. A run on the whole forward has the same result
//!   as one on the allowance, and charges exactly the price;
//! - `allowance < price <= forward`: the call needs gas the consumer held back. This is what the
//!   withheld part of a frame's regular gas calls a crossing;
//! - `price > forward`: the call runs out of gas on its whole forward too. It is a plain
//!   out-of-gas, not a crossing, although a run on the allowance runs out of gas just the same,
//!   so the run alone cannot tell this case from the one before.
//!
//! Charging the price on a `GasTracker` that holds the forward, with its spendable part limited
//! to the allowance (`limit_spendable`), fails in the last two cases and records a crossing
//! (`withheld_crossing`) in the middle one only, as the same regular charge in a frame would. A
//! consumer that records the call's crossing on the frame can let the tracker build the record.
//!
//! # Keeping the prices in step with the runs
//!
//! Most price functions read the constants the runs charge. [`ecrecover`], [`sha256`] and
//! [`ripemd160`] repeat literals the runs keep as locals (`3_000`, `60`/`12`, `600`/`120`), and
//! [`blake2f`] the run's private input length and per-round price (213 bytes, 1 gas), so a
//! change to one of those run bodies does not reach the price function. The differential test,
//! `required_gas_matches_every_builtin_run` in `tests/required_gas.rs`, runs every built-in at
//! the limits around its price, on edge and random inputs, and is the guard: a change to a run's
//! gas that its price function does not follow fails it. It runs under every backend the crate
//! is tested with.

use crate::{
    bls12_381_const::{
        DISCOUNT_TABLE_G1_MSM, DISCOUNT_TABLE_G2_MSM, G1_ADD_BASE_GAS_FEE, G1_MSM_BASE_GAS_FEE,
        G1_MSM_INPUT_LENGTH, G2_ADD_BASE_GAS_FEE, G2_MSM_BASE_GAS_FEE, G2_MSM_INPUT_LENGTH,
        MAP_FP2_TO_G2_BASE_GAS_FEE, MAP_FP_TO_G1_BASE_GAS_FEE, PAIRING_INPUT_LENGTH,
        PAIRING_MULTIPLIER_BASE, PAIRING_OFFSET_BASE,
    },
    bls12_381_utils::msm_required_gas,
    bn254::{
        add::{BYZANTIUM_ADD_GAS_COST, ISTANBUL_ADD_GAS_COST},
        mul::{BYZANTIUM_MUL_GAS_COST, ISTANBUL_MUL_GAS_COST},
        pair::{
            BYZANTIUM_PAIR_BASE, BYZANTIUM_PAIR_PER_POINT, ISTANBUL_PAIR_BASE,
            ISTANBUL_PAIR_PER_POINT,
        },
        PAIR_ELEMENT_LEN,
    },
    calc_linear_cost,
    identity::{IDENTITY_BASE, IDENTITY_PER_WORD},
    kzg_point_evaluation, modexp,
    secp256r1::{P256VERIFY_BASE_GAS_FEE, P256VERIFY_BASE_GAS_FEE_OSAKA},
    Precompile,
};
use core::cmp::max;
use primitives::U256;

/// A precompile's price function: the gas a call with the given input needs.
///
/// See [`Precompile::required_gas`] for the contract it must keep.
pub type PrecompileGasFn = fn(&[u8]) -> u64;

impl Precompile {
    /// Returns the precompile with `required_gas` as its price function, which
    /// [`required_gas`](Self::required_gas) then answers with.
    ///
    /// `required_gas` must keep the contract of [`required_gas`](Self::required_gas) against this
    /// precompile's run for every input; nothing checks it at run time.
    ///
    /// A precompile that wraps one of this crate's runs behind a check of its own, such as an
    /// input size limit, prices itself from the price function of the run it wraps, and prices
    /// the inputs its own check refuses at `0`, since they fail the same way at every gas limit:
    ///
    /// ```
    /// use revm_precompile::{bn254, required_gas, Precompile, PrecompileId};
    /// # use revm_precompile::PrecompileResult;
    ///
    /// const MAX_INPUT_SIZE: usize = 4 * 192;
    ///
    /// fn limited_pair_price(input: &[u8]) -> u64 {
    ///     if input.len() > MAX_INPUT_SIZE {
    ///         0
    ///     } else {
    ///         required_gas::bn254_pair_istanbul(input)
    ///     }
    /// }
    /// # // Refuses an input above the limit, then runs `bn254::run_pair` with Istanbul's gas.
    /// # fn limited_pair(_input: &[u8], _gas_limit: u64, _reservoir: u64) -> PrecompileResult {
    /// #     unimplemented!()
    /// # }
    ///
    /// const LIMITED_PAIR: Precompile =
    ///     Precompile::new(PrecompileId::Bn254Pairing, bn254::pair::ADDRESS, limited_pair)
    ///         .with_required_gas(limited_pair_price);
    ///
    /// assert_eq!(LIMITED_PAIR.required_gas(&[0; 5 * 192]), Some(0));
    /// assert_eq!(LIMITED_PAIR.required_gas(&[0; 4 * 192]), Some(4 * 34_000 + 45_000));
    /// ```
    ///
    /// Forwarding the wrapped price unchanged would price a refused input above `0`, and a limit
    /// below that price would then fail with the wrapper's refusal, not out of gas. A wrapper's
    /// price function needs the same differential test as this crate's (`tests/required_gas.rs`):
    /// its run at the limits below, at and above the price, on inputs its own check accepts and
    /// refuses.
    #[inline]
    pub const fn with_required_gas(mut self, required_gas: PrecompileGasFn) -> Self {
        self.required_gas = Some(required_gas);
        self
    }

    /// Returns the gas a call with `input` needs, without running the precompile, or `None` when
    /// the precompile has no price function.
    ///
    /// `Some(price)` guarantees, for every gas limit `g`: [`execute`](Self::execute) with `g`
    /// halts with [`PrecompileHalt::OutOfGas`](crate::PrecompileHalt::OutOfGas) if and only if
    /// `g < price`, and with `g >= price` its result is the same for every `g`, a success or a
    /// revert using exactly `price`. An input a check refuses before any gas check prices at `0`.
    /// See the [module documentation](self) for the malformed inputs of each precompile.
    #[inline]
    pub fn required_gas(&self, input: &[u8]) -> Option<u64> {
        self.required_gas.map(|required_gas| required_gas(input))
    }
}

/// `ECRECOVER`: 3,000, whatever the input.
pub const fn ecrecover(_input: &[u8]) -> u64 {
    3_000
}

/// `SHA256`: 60, and 12 per word of input.
pub const fn sha256(input: &[u8]) -> u64 {
    calc_linear_cost(input.len(), 60, 12)
}

/// `RIPEMD160`: 600, and 120 per word of input.
pub const fn ripemd160(input: &[u8]) -> u64 {
    calc_linear_cost(input.len(), 600, 120)
}

/// `IDENTITY`: 15, and 3 per word of input.
pub const fn identity(input: &[u8]) -> u64 {
    calc_linear_cost(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD)
}

/// `MODEXP` under the Byzantium schedule (EIP-198).
pub fn modexp_byzantium(input: &[u8]) -> u64 {
    modexp_price::<false>(input, 0, modexp::byzantium_gas_calc)
}

/// `MODEXP` under the Berlin schedule (EIP-2565).
pub fn modexp_berlin(input: &[u8]) -> u64 {
    modexp_price::<false>(input, 200, modexp::berlin_gas_calc)
}

/// `MODEXP` under the Osaka schedule and size limits (EIP-7823, EIP-7883).
pub fn modexp_osaka(input: &[u8]) -> u64 {
    modexp_price::<true>(input, 500, modexp::osaka_gas_calc)
}

/// Prices a `MODEXP` call with the run's own header parsing: [`modexp::run_inner`] reads the
/// header, checks the sizes and prices the call exactly as a run does, and `gas_calc` is handed
/// to it as the price, recorded and then refused, so the run stops at its gas check before any
/// exponentiation. A header refused between the two gas checks never reaches `gas_calc`, and
/// prices at `min_gas`.
fn modexp_price<const OSAKA: bool>(
    input: &[u8],
    min_gas: u64,
    gas_calc: fn(u64, u64, u64, &U256) -> u64,
) -> u64 {
    let mut price = min_gas;
    // A limit the first gas check passes whatever `min_gas`, and a price above it, so the second
    // gas check fails.
    let _ = modexp::run_inner::<_, OSAKA>(
        input,
        u64::MAX - 1,
        min_gas,
        |base_len, exp_len, mod_len, exp_highp| {
            price = max(min_gas, gas_calc(base_len, exp_len, mod_len, exp_highp));
            u64::MAX
        },
    );
    price
}

/// `BN254_ADD` under the Byzantium schedule: 500.
pub const fn bn254_add_byzantium(_input: &[u8]) -> u64 {
    BYZANTIUM_ADD_GAS_COST
}

/// `BN254_ADD` under the Istanbul schedule (EIP-1108): 150.
pub const fn bn254_add_istanbul(_input: &[u8]) -> u64 {
    ISTANBUL_ADD_GAS_COST
}

/// `BN254_MUL` under the Byzantium schedule: 40,000.
pub const fn bn254_mul_byzantium(_input: &[u8]) -> u64 {
    BYZANTIUM_MUL_GAS_COST
}

/// `BN254_MUL` under the Istanbul schedule (EIP-1108): 6,000.
pub const fn bn254_mul_istanbul(_input: &[u8]) -> u64 {
    ISTANBUL_MUL_GAS_COST
}

/// `BN254_PAIRING` under the Byzantium schedule.
pub const fn bn254_pair_byzantium(input: &[u8]) -> u64 {
    bn254_pair(input, BYZANTIUM_PAIR_PER_POINT, BYZANTIUM_PAIR_BASE)
}

/// `BN254_PAIRING` under the Istanbul schedule (EIP-1108).
pub const fn bn254_pair_istanbul(input: &[u8]) -> u64 {
    bn254_pair(input, ISTANBUL_PAIR_PER_POINT, ISTANBUL_PAIR_BASE)
}

/// `BN254_PAIRING` at `pair_per_point_cost` per whole 192-byte pair and `pair_base_cost`, as
/// [`bn254::run_pair`](crate::bn254::run_pair) charges it. A length that is not a multiple of 192
/// is refused after the gas check, so it prices its whole pairs.
pub const fn bn254_pair(input: &[u8], pair_per_point_cost: u64, pair_base_cost: u64) -> u64 {
    (input.len() / PAIR_ELEMENT_LEN) as u64 * pair_per_point_cost + pair_base_cost
}

/// `BLAKE2F` (EIP-152): one gas per round. An input that is not 213 bytes is refused before the
/// rounds are read, so it prices at `0`.
pub const fn blake2f(input: &[u8]) -> u64 {
    if input.len() != 213 {
        return 0;
    }
    u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as u64
}

/// `KZG_POINT_EVALUATION` (EIP-4844): 50,000, whatever the input.
pub const fn kzg_point_evaluation(_input: &[u8]) -> u64 {
    kzg_point_evaluation::GAS_COST
}

/// `BLS12_G1ADD` (EIP-2537): 375, whatever the input.
pub const fn bls12_g1_add(_input: &[u8]) -> u64 {
    G1_ADD_BASE_GAS_FEE
}

/// `BLS12_G2ADD` (EIP-2537): 600, whatever the input.
pub const fn bls12_g2_add(_input: &[u8]) -> u64 {
    G2_ADD_BASE_GAS_FEE
}

/// `BLS12_G1MSM` (EIP-2537), discounted by the number of pairs. A length that is not a positive
/// multiple of 160 is refused before pricing, so it prices at `0`.
pub fn bls12_g1_msm(input: &[u8]) -> u64 {
    let len = input.len();
    if len == 0 || !len.is_multiple_of(G1_MSM_INPUT_LENGTH) {
        return 0;
    }
    msm_required_gas(
        len / G1_MSM_INPUT_LENGTH,
        &DISCOUNT_TABLE_G1_MSM,
        G1_MSM_BASE_GAS_FEE,
    )
}

/// `BLS12_G2MSM` (EIP-2537), discounted by the number of pairs. A length that is not a positive
/// multiple of 288 is refused before pricing, so it prices at `0`.
pub fn bls12_g2_msm(input: &[u8]) -> u64 {
    let len = input.len();
    if len == 0 || !len.is_multiple_of(G2_MSM_INPUT_LENGTH) {
        return 0;
    }
    msm_required_gas(
        len / G2_MSM_INPUT_LENGTH,
        &DISCOUNT_TABLE_G2_MSM,
        G2_MSM_BASE_GAS_FEE,
    )
}

/// `BLS12_PAIRING_CHECK` (EIP-2537): 37,700, and 32,600 per pair. A length that is not a
/// positive multiple of 384 is refused before pricing, so it prices at `0`.
pub const fn bls12_pairing(input: &[u8]) -> u64 {
    let len = input.len();
    if len == 0 || !len.is_multiple_of(PAIRING_INPUT_LENGTH) {
        return 0;
    }
    PAIRING_MULTIPLIER_BASE * (len / PAIRING_INPUT_LENGTH) as u64 + PAIRING_OFFSET_BASE
}

/// `BLS12_MAP_FP_TO_G1` (EIP-2537): 5,500, whatever the input.
pub const fn bls12_map_fp_to_g1(_input: &[u8]) -> u64 {
    MAP_FP_TO_G1_BASE_GAS_FEE
}

/// `BLS12_MAP_FP2_TO_G2` (EIP-2537): 23,800, whatever the input.
pub const fn bls12_map_fp2_to_g2(_input: &[u8]) -> u64 {
    MAP_FP2_TO_G2_BASE_GAS_FEE
}

/// `P256VERIFY` (RIP-7212): 3,450, whatever the input.
pub const fn p256verify(_input: &[u8]) -> u64 {
    P256VERIFY_BASE_GAS_FEE
}

/// `P256VERIFY` under Osaka (EIP-7951): 6,900, whatever the input.
pub const fn p256verify_osaka(_input: &[u8]) -> u64 {
    P256VERIFY_BASE_GAS_FEE_OSAKA
}
