//! `Precompile::required_gas` against each precompile's own run.
//!
//! For every precompile the crate defines, on edge inputs and on random ones, the price
//! `required_gas` answers is checked against runs of the precompile: every gas limit below the
//! price halts out of gas, the price itself and every limit above it give one result, and a
//! success or a revert uses exactly the price. Each precompile gets inputs its run accepts, so the
//! gas a successful run uses is compared, and inputs its checks refuse before and after its gas
//! check, so the malformed-input answers are compared too.

use context_interface::cfg::gas::GasTracker;
use rand::{rngs::StdRng, Rng, SeedableRng};
use revm_precompile::{
    blake2, bls12_381, bn254, hash, identity, kzg_point_evaluation, modexp,
    primitives::{hex, U256},
    secp256k1, secp256r1, Precompile, PrecompileHalt, PrecompileId, PrecompileOutput,
    PrecompileResult, PrecompileSpecId, PrecompileStatus, Precompiles,
};

/// Prices above this are checked on the insufficient-gas side only. Running them costs real time
/// (2^32 `BLAKE2F` rounds, an exponent of a thousand bytes) and tests nothing the cheaper inputs
/// do not.
const RUN_LIMIT: u64 = 1_000_000;

/// Random inputs per precompile, on top of its edge inputs.
const RANDOM_INPUTS: usize = 96;

/// Input lengths around every element size the precompiles read.
const EDGE_LENGTHS: [usize; 40] = [
    0, 1, 4, 31, 32, 33, 63, 64, 65, 95, 96, 97, 127, 128, 129, 159, 160, 161, 191, 192, 193, 212,
    213, 214, 255, 256, 257, 287, 288, 289, 383, 384, 385, 575, 576, 577, 767, 768, 769, 1_024,
];

/// Every precompile the crate defines, each schedule of it once.
fn every_builtin() -> Vec<Precompile> {
    let mut all = vec![
        secp256k1::ECRECOVER,
        hash::SHA256,
        hash::RIPEMD160,
        identity::FUN,
        modexp::BYZANTIUM,
        modexp::BERLIN,
        modexp::OSAKA,
        bn254::add::BYZANTIUM,
        bn254::add::ISTANBUL,
        bn254::mul::BYZANTIUM,
        bn254::mul::ISTANBUL,
        bn254::pair::BYZANTIUM,
        bn254::pair::ISTANBUL,
        blake2::FUN,
        kzg_point_evaluation::POINT_EVALUATION,
        secp256r1::P256VERIFY,
        secp256r1::P256VERIFY_OSAKA,
    ];
    all.extend(bls12_381::precompiles());
    all
}

/// Every precompile set a spec selects.
const SPECS: [PrecompileSpecId; 7] = [
    PrecompileSpecId::HOMESTEAD,
    PrecompileSpecId::BYZANTIUM,
    PrecompileSpecId::ISTANBUL,
    PrecompileSpecId::BERLIN,
    PrecompileSpecId::CANCUN,
    PrecompileSpecId::PRAGUE,
    PrecompileSpecId::OSAKA,
];

/// What the runs of one precompile showed.
#[derive(Debug, Default)]
struct Seen {
    /// Runs at the price that succeeded or reverted, whose gas used was compared.
    used_the_price: usize,
    /// Runs at the price that a check after the gas check failed.
    failed_after_the_gas_check: usize,
    /// Inputs a check before any gas check refused: priced at 0 and failing at every limit.
    refused_before_pricing: usize,
    /// Inputs priced above [`RUN_LIMIT`], checked below the price only.
    not_run: usize,
}

const fn is_out_of_gas(output: &PrecompileOutput) -> bool {
    matches!(
        output.status,
        PrecompileStatus::Halt(PrecompileHalt::OutOfGas)
    )
}

fn run(precompile: &Precompile, input: &[u8], gas_limit: u64) -> PrecompileOutput {
    let result: PrecompileResult = precompile.execute(input, gas_limit, 0);
    result.expect("no built-in precompile fails fatally")
}

/// Checks the price `precompile` answers for `input` against its runs.
fn check(precompile: &Precompile, input: &[u8], seen: &mut Seen) {
    let name = precompile.id().name();
    let price = precompile
        .required_gas(input)
        .unwrap_or_else(|| panic!("{name} has no price function"));
    let shown = || hex::encode(&input[..input.len().min(64)]);

    // Below the price: out of gas, at every limit.
    for limit in [0, price / 2, price.saturating_sub(1)] {
        if limit < price {
            assert!(
                is_out_of_gas(&run(precompile, input, limit)),
                "{name}: a limit of {limit} below the price {price} must run out of gas \
                 (input {} bytes, {}..)",
                input.len(),
                shown(),
            );
        }
    }
    if price > RUN_LIMIT {
        seen.not_run += 1;
        return;
    }

    // At the price and above it: never out of gas, one result, and a success uses the price.
    let at_price = run(precompile, input, price);
    assert!(
        !is_out_of_gas(&at_price),
        "{name}: the price {price} must be enough (input {} bytes, {}..)",
        input.len(),
        shown(),
    );
    for limit in [
        price + 1,
        price.saturating_mul(2).saturating_add(7),
        u64::MAX,
    ] {
        assert_eq!(
            run(precompile, input, limit),
            at_price,
            "{name}: a limit of {limit} must give the result of the price {price} \
             (input {} bytes, {}..)",
            input.len(),
            shown(),
        );
    }
    if at_price.is_success() || at_price.is_revert() {
        assert_eq!(
            at_price.gas_used,
            price,
            "{name}: a success must use the price (input {} bytes, {}..)",
            input.len(),
            shown(),
        );
        seen.used_the_price += 1;
    } else if price == 0 {
        seen.refused_before_pricing += 1;
    } else {
        seen.failed_after_the_gas_check += 1;
    }
}

fn random_bytes(rng: &mut StdRng, len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    rng.fill(&mut bytes[..]);
    bytes
}

/// A `MODEXP` input: the three lengths as 32-byte words, then `body` bytes of random operands.
fn modexp_input(
    rng: &mut StdRng,
    base_len: U256,
    exp_len: U256,
    mod_len: U256,
    body: usize,
) -> Vec<u8> {
    let mut input = Vec::with_capacity(96 + body);
    for len in [base_len, exp_len, mod_len] {
        input.extend_from_slice(&len.to_be_bytes::<32>());
    }
    input.extend(random_bytes(rng, body));
    input
}

/// A `BLAKE2F` input of the right length with `rounds` and a final-block flag of `flag`.
fn blake2f_input(rng: &mut StdRng, rounds: u32, flag: u8) -> Vec<u8> {
    let mut input = random_bytes(rng, 213);
    input[..4].copy_from_slice(&rounds.to_be_bytes());
    input[212] = flag;
    input
}

/// The BN254 G1 generator, `(1, 2)`.
fn bn254_generator() -> Vec<u8> {
    let mut point = vec![0; 64];
    point[31] = 1;
    point[63] = 2;
    point
}

/// A KZG point evaluation input the proof verifies: the `verify_kzg_proof_case_correct_proof_4_4`
/// vector of the c-kzg-4844 test suite, which the crate's own test uses.
fn kzg_valid_input() -> Vec<u8> {
    let commitment = hex::decode("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").unwrap();
    let z =
        hex::decode("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").unwrap();
    let y =
        hex::decode("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").unwrap();
    let proof = hex::decode("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").unwrap();
    let versioned_hash = kzg_point_evaluation::kzg_to_versioned_hash(&commitment);
    [versioned_hash.to_vec(), z, y, commitment, proof].concat()
}

/// Inputs made for `id`: ones its run accepts, and ones its checks refuse before and after its
/// gas check.
fn made_inputs(id: &PrecompileId, rng: &mut StdRng) -> Vec<Vec<u8>> {
    let mut inputs = Vec::new();
    match id {
        PrecompileId::ModExp => {
            let small = |n: usize| U256::from(n);
            // Sizes around the words the price reads, the exponent's high word above all.
            for (base, exp, modulus) in [
                (0, 0, 0),
                (0, 0, 1),
                (1, 1, 1),
                (1, 0, 1),
                (32, 32, 32),
                (0, 32, 32),
                (32, 33, 32),
                (1, 64, 1),
                (64, 64, 64),
                (100, 3, 100),
                (1_024, 1, 1_024),
            ] {
                let body = base + exp + modulus;
                inputs.push(modexp_input(
                    rng,
                    small(base),
                    small(exp),
                    small(modulus),
                    body,
                ));
                // A short body is right-padded with zeros.
                inputs.push(modexp_input(
                    rng,
                    small(base),
                    small(exp),
                    small(modulus),
                    body / 2,
                ));
            }
            // An exponent that is all zeros, and one whose high word is set.
            let mut zero_exp = modexp_input(rng, small(1), small(32), small(1), 34);
            zero_exp[96 + 1..96 + 33].fill(0);
            inputs.push(zero_exp);
            let mut high_exp = modexp_input(rng, small(1), small(40), small(1), 42);
            high_exp[96 + 1] = 0xff;
            inputs.push(high_exp);
            // Sizes EIP-7823 refuses under Osaka, between the two gas checks.
            for (base, exp, modulus) in [(1_025, 1, 1), (1, 1_025, 1), (1, 1, 1_025)] {
                inputs.push(modexp_input(
                    rng,
                    small(base),
                    small(exp),
                    small(modulus),
                    8,
                ));
            }
            // Sizes that do not fit a `usize`, refused everywhere, and an exponent size that
            // saturates the price.
            let huge = U256::from(u64::MAX) + U256::from(1);
            inputs.push(modexp_input(rng, huge, small(1), small(1), 8));
            inputs.push(modexp_input(rng, small(1), small(1), U256::MAX, 8));
            inputs.push(modexp_input(rng, small(1), huge, small(1), 8));
            inputs.push(modexp_input(rng, small(0), U256::MAX, small(0), 8));
            // Headers cut short, read as right-padded with zeros.
            for len in [0, 1, 31, 32, 63, 64, 95] {
                let mut header = modexp_input(rng, small(1), small(1), small(1), 0);
                header.truncate(len);
                inputs.push(header);
            }
            for _ in 0..RANDOM_INPUTS {
                let base = rng.random_range(0..=48);
                let exp = rng.random_range(0..=48);
                let modulus = rng.random_range(0..=48);
                let body = rng.random_range(0..=base + exp + modulus);
                inputs.push(modexp_input(
                    rng,
                    small(base),
                    small(exp),
                    small(modulus),
                    body,
                ));
            }
        }
        PrecompileId::Bn254Add => {
            let generator = bn254_generator();
            inputs.push([generator.clone(), generator.clone()].concat());
            inputs.push(generator);
        }
        PrecompileId::Bn254Mul => {
            let generator = bn254_generator();
            for _ in 0..4 {
                inputs.push([bn254_generator(), random_bytes(rng, 32)].concat());
            }
            inputs.push(generator);
        }
        PrecompileId::Bn254Pairing => {
            for pairs in 0..=3 {
                inputs.push(vec![0; 192 * pairs]);
            }
        }
        PrecompileId::Blake2F => {
            for rounds in [0, 1, 12, 1_000, u32::MAX] {
                for flag in [0, 1, 2, 0xff] {
                    inputs.push(blake2f_input(rng, rounds, flag));
                }
            }
            for _ in 0..RANDOM_INPUTS / 4 {
                let rounds = rng.random_range(0..=2_000);
                let flag = rng.random_range(0..=2);
                inputs.push(blake2f_input(rng, rounds, flag));
            }
        }
        PrecompileId::KzgPointEvaluation => {
            let valid = kzg_valid_input();
            let mut mismatched = valid.clone();
            mismatched[0] = 0x02;
            let mut wrong_proof = valid.clone();
            wrong_proof[191] ^= 1;
            inputs.extend([valid, mismatched, wrong_proof]);
        }
        PrecompileId::Bls12G1Msm | PrecompileId::Bls12G2Msm => {
            let point = if *id == PrecompileId::Bls12G1Msm {
                128
            } else {
                256
            };
            // Points at infinity with random scalars, which every backend accepts.
            for pairs in 1..=3 {
                let mut input = Vec::new();
                for _ in 0..pairs {
                    input.extend(vec![0; point]);
                    input.extend(random_bytes(rng, 32));
                }
                inputs.push(input);
            }
        }
        PrecompileId::Bls12G1Add | PrecompileId::Bls12G2Add => {
            // Two points at infinity, and one input a byte short or long.
            let len = if *id == PrecompileId::Bls12G1Add {
                256
            } else {
                512
            };
            inputs.extend([vec![0; len], vec![0; len - 1], vec![0; len + 1]]);
        }
        PrecompileId::Bls12Pairing => {
            for pairs in 1..=2 {
                inputs.push(vec![0; 384 * pairs]);
            }
        }
        PrecompileId::Bls12MapFpToGp1 | PrecompileId::Bls12MapFp2ToGp2 => {
            // Field elements below the modulus (its top byte is 0x1a), each padded to 64 bytes.
            let elements = if *id == PrecompileId::Bls12MapFpToGp1 {
                1
            } else {
                2
            };
            for _ in 0..4 {
                let mut input = Vec::new();
                for _ in 0..elements {
                    let mut element = vec![0; 16];
                    element.push(rng.random_range(0..0x1a));
                    element.extend(random_bytes(rng, 47));
                    input.extend(element);
                }
                inputs.push(input);
            }
        }
        PrecompileId::P256Verify => {
            for _ in 0..4 {
                inputs.push(random_bytes(rng, 160));
            }
        }
        _ => {}
    }
    inputs
}

/// Edge-length inputs, zeroed and random, then random ones: every precompile gets these on top of
/// its own.
fn shared_inputs(rng: &mut StdRng) -> Vec<Vec<u8>> {
    let mut inputs = Vec::new();
    for len in EDGE_LENGTHS {
        inputs.push(vec![0; len]);
        inputs.push(random_bytes(rng, len));
    }
    for _ in 0..RANDOM_INPUTS {
        let len = rng.random_range(0..=1_200);
        inputs.push(random_bytes(rng, len));
    }
    inputs
}

/// The differential test: every built-in precompile, on its made inputs and the shared ones.
///
/// It is the guard of the price functions that repeat a literal of their run (`ecrecover`,
/// `sha256`, `ripemd160`, `blake2f`): a pick that changes the run's gas and not the price fails
/// here.
#[test]
fn required_gas_matches_every_builtin_run() {
    for precompile in every_builtin() {
        let name = precompile.id().name().to_owned();
        let mut rng = StdRng::seed_from_u64(0x5eed ^ precompile.address().0[19] as u64);
        let mut seen = Seen::default();
        let mut inputs = made_inputs(precompile.id(), &mut rng);
        inputs.extend(shared_inputs(&mut rng));
        for input in &inputs {
            check(&precompile, input, &mut seen);
        }
        assert!(
            seen.used_the_price > 0,
            "{name}: no input succeeded, so the gas a run uses was never compared: {seen:?}"
        );
        println!("{name}: {} inputs, {seen:?}", inputs.len());
    }
}

/// Each malformed input answers what the module documentation says it does.
#[test]
fn malformed_inputs_price_as_documented() {
    // A check before any gas check: the input needs no gas to fail.
    for (precompile, input) in [
        (blake2::FUN, vec![0; 212]),
        (bls12_381::g1_msm::PRECOMPILE, vec![0; 159]),
        (bls12_381::g1_msm::PRECOMPILE, vec![]),
        (bls12_381::g2_msm::PRECOMPILE, vec![0; 289]),
        (bls12_381::pairing::PRECOMPILE, vec![0; 385]),
    ] {
        assert_eq!(precompile.required_gas(&input), Some(0));
        let output = run(&precompile, &input, 0);
        assert!(output.is_halt() && !is_out_of_gas(&output));
    }

    // A check after the gas check: the input needs what that gas check asks for.
    assert_eq!(
        kzg_point_evaluation::POINT_EVALUATION.required_gas(&[0; 5]),
        Some(50_000)
    );
    assert_eq!(bn254::pair::ISTANBUL.required_gas(&[0; 193]), Some(79_000));
    assert_eq!(
        bls12_381::g1_add::PRECOMPILE.required_gas(&[0; 3]),
        Some(375)
    );
    assert_eq!(
        blake2::FUN.required_gas(&blake2f_input(&mut StdRng::seed_from_u64(1), 7, 2)),
        Some(7)
    );

    // `MODEXP` sizes refused between its two gas checks: the first gas check's minimum.
    let mut rng = StdRng::seed_from_u64(2);
    let too_big = modexp_input(&mut rng, U256::from(1_025), U256::from(1), U256::from(1), 8);
    assert_eq!(modexp::OSAKA.required_gas(&too_big), Some(500));
    let unaddressable = modexp_input(&mut rng, U256::MAX, U256::from(1), U256::from(1), 8);
    assert_eq!(modexp::OSAKA.required_gas(&unaddressable), Some(500));
    assert_eq!(modexp::BERLIN.required_gas(&unaddressable), Some(200));
    assert_eq!(modexp::BYZANTIUM.required_gas(&unaddressable), Some(0));
}

/// Every precompile of every set answers, and each set carries the schedule of its spec: the
/// `MODEXP` of Byzantium, Berlin and Osaka tell apart a header whose sizes do not fit.
#[test]
fn every_set_prices_with_its_own_schedule() {
    for spec in SPECS {
        for precompile in Precompiles::new(spec).inner().values() {
            assert!(
                precompile.required_gas(&[]).is_some(),
                "{} in {spec:?} has no price function",
                precompile.id().name()
            );
        }
    }

    let unaddressable = {
        let mut input = U256::MAX.to_be_bytes::<32>().to_vec();
        input.extend(U256::from(1).to_be_bytes::<32>());
        input.extend(U256::from(1).to_be_bytes::<32>());
        input
    };
    let modexp_price = |spec| {
        Precompiles::new(spec)
            .get(modexp::OSAKA.address())
            .and_then(|precompile| precompile.required_gas(&unaddressable))
    };
    assert_eq!(modexp_price(PrecompileSpecId::BYZANTIUM), Some(0));
    assert_eq!(modexp_price(PrecompileSpecId::BERLIN), Some(200));
    assert_eq!(modexp_price(PrecompileSpecId::PRAGUE), Some(200));
    assert_eq!(modexp_price(PrecompileSpecId::OSAKA), Some(500));

    let p256 = |spec| {
        Precompiles::new(spec)
            .get(secp256r1::P256VERIFY_OSAKA.address())
            .and_then(|precompile| precompile.required_gas(&[]))
    };
    assert_eq!(p256(PrecompileSpecId::OSAKA), Some(6_900));
    assert_eq!(secp256r1::P256VERIFY.required_gas(&[]), Some(3_450));
}

/// A call held to an allowance below its forward crosses the allowance exactly when
/// `allowance < price <= forward`, and charging the price on a tracker that holds the forward,
/// limited to the allowance, records a crossing in exactly those cases. A run on the allowance
/// runs out of gas whenever `allowance < price`, so it would take a call priced above its whole
/// forward, a plain out-of-gas, for a crossing; the price tells them apart.
#[test]
fn a_held_call_crosses_exactly_when_its_price_is_within_the_forward() {
    let inputs: Vec<Vec<u8>> = [0, 1, 32, 64, 96, 128, 160, 192, 213, 256, 288, 384, 480]
        .iter()
        .map(|&len| (0..len).map(|i| (i * 7 + 3) as u8).collect())
        .collect();
    let (mut cases, mut crossings, mut beyond_the_forward) = (0, 0, 0);
    for precompile in every_builtin() {
        let name = precompile.id().name();
        for input in &inputs {
            let price = precompile.required_gas(input).unwrap();
            if price > RUN_LIMIT {
                continue;
            }
            for forward in [price + 500, price.saturating_sub(1)] {
                if forward == 0 {
                    continue;
                }
                for allowance in [0, price / 2, price.saturating_sub(1), price, forward] {
                    if allowance > forward {
                        continue;
                    }
                    cases += 1;
                    let crossed = allowance < price && price <= forward;

                    let mut tracker = GasTracker::new(forward, forward, 0);
                    tracker.limit_spendable(allowance);
                    let paid = tracker.record_regular_cost(price);
                    assert_eq!(paid, price <= allowance, "{name}: {allowance} of {forward}");
                    assert_eq!(
                        tracker.withheld_crossing().is_some(),
                        crossed,
                        "{name}: price {price}, allowance {allowance} of {forward}"
                    );

                    let out_of_gas = is_out_of_gas(&run(&precompile, input, allowance));
                    assert_eq!(out_of_gas, allowance < price, "{name}: {allowance}");
                    if crossed {
                        crossings += 1;
                    } else if out_of_gas && allowance < forward {
                        // Runs out of gas on the allowance, but needs more than the forward.
                        assert!(price > forward, "{name}");
                        beyond_the_forward += 1;
                    }
                }
            }
        }
    }
    assert!(crossings > 0 && beyond_the_forward > 0);
    println!(
        "{cases} held calls: {crossings} crossings; {beyond_the_forward} priced above the \
         forward, which run out of gas on the allowance too"
    );
}

const fn always_out_of_gas(_input: &[u8], _gas_limit: u64, reservoir: u64) -> PrecompileResult {
    Ok(PrecompileOutput::halt(PrecompileHalt::OutOfGas, reservoir))
}

/// A precompile defined outside the crate has no price until it sets one, and then answers it.
#[test]
fn a_precompile_without_a_price_function_answers_none() {
    let address = revm_precompile::u64_to_address(0x100);
    let custom = Precompile::new(PrecompileId::custom("custom"), address, always_out_of_gas);
    assert_eq!(custom.required_gas(&[1, 2, 3]), None);

    let from_tuple = Precompile::from((
        PrecompileId::custom("custom"),
        address,
        always_out_of_gas as revm_precompile::PrecompileFn,
    ));
    assert_eq!(from_tuple.required_gas(&[]), None);

    let priced = custom.with_required_gas(|input| input.len() as u64 * 10);
    assert_eq!(priced.required_gas(&[1, 2, 3]), Some(30));
    assert_eq!(priced.id(), &PrecompileId::custom("custom"));
    assert_eq!(priced.address(), &address);
}
