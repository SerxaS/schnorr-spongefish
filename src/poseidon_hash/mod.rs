pub mod params;
/// Native sponge implementation
pub mod sponge;
use self::params::hasher::{RoundParams, Sbox, poseidon_bn254_5x5::Params};
use ark_ff::Field;

/// Constructs objects.
#[derive(Debug, Clone)]
pub struct Poseidon<F: Field> {
    /// Constructs an array for the inputs.
    inputs: [F; 5],
}

impl<F: Field> Poseidon<F> {
    /// Create the objects.
    pub fn new(inputs: [F; 5]) -> Self {
        Poseidon { inputs }
    }

    /// The Hades Design Strategy for Hashing.
    /// Mixing rounds with half-full S-box layers and
    /// rounds with partial S-box layers.
    /// More detailed explanation for
    /// The Round Function (TRF) and Hades:
    /// https://eprint.iacr.org/2019/458.pdf#page=5
    pub fn permute(&self) -> [F; 5] {
        let full_rounds = Params::<F>::full_rounds();
        let half_full_rounds = full_rounds / 2;
        let partial_rounds = Params::<F>::partial_rounds();
        let round_constants = Params::<F>::round_constants();
        let total_count = Params::<F>::round_constants_count();

        let first_round_end = half_full_rounds * 5;
        let first_round_constants = &round_constants[0..first_round_end];

        let second_round_end = first_round_end + partial_rounds * 5;
        let second_round_constants = &round_constants[first_round_end..second_round_end];

        let third_round_constants = &round_constants[second_round_end..total_count];

        let mut state = self.inputs;
        for round in 0..half_full_rounds {
            let round_consts = Params::<F>::load_round_constants(round, first_round_constants);
            // 1. step for the TRF.
            // AddRoundConstants step.
            state = Params::<F>::apply_round_constants(&state, &round_consts);
            // Applying S-boxes for the full round.
            for state in state.iter_mut().take(5) {
                // 2. step for the TRF.
                // SubWords step.
                *state = Params::<F>::sbox_f(*state);
            }
            // 3. step for the TRF.
            // MixLayer step.
            state = Params::<F>::apply_mds(&state);
        }

        for round in 0..partial_rounds {
            let round_consts = Params::<F>::load_round_constants(round, second_round_constants);
            // 1. step for the TRF.
            // AddRoundConstants step.
            state = Params::<F>::apply_round_constants(&state, &round_consts);
            // Applying single S-box for the partial round.
            // 2. step for the TRF.
            // SubWords step, denoted by S-box.
            state[0] = Params::<F>::sbox_f(state[0]);
            // 3. step for the TRF.
            // MixLayer step.
            state = Params::<F>::apply_mds(&state);
        }

        for round in 0..half_full_rounds {
            let round_consts = Params::<F>::load_round_constants(round, third_round_constants);
            // 1. step for the TRF.
            // AddRoundConstants step.
            state = Params::<F>::apply_round_constants(&state, &round_consts);
            // Applying S-boxes for the full round.
            for state in state.iter_mut().take(5) {
                // 2. step for the TRF.
                // SubWords step, denoted by S-box.
                *state = Params::<F>::sbox_f(*state);
            }
            // 3. step for the TRF.
            // MixLayer step.
            state = Params::<F>::apply_mds(&state);
        }

        state
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::poseidon_hash::params::hasher::hex_to_field;
    use ark_bn254::Fr;

    type TestPoseidon = Poseidon<Fr>;

    #[test]
    fn test_native_poseidon_5x5() {
        // Testing 5x5 input.
        let inputs: [Fr; 5] = [
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            "0x0000000000000000000000000000000000000000000000000000000000000004",
        ]
        .map(|n| hex_to_field(n));

        let outputs: [Fr; 5] = [
            "0x299c867db6c1fdd79dcefa40e4510b9837e60ebb1ce0663dbaa525df65250465",
            "0x1148aaef609aa338b27dafd89bb98862d8bb2b429aceac47d86206154ffe053d",
            "0x24febb87fed7462e23f6665ff9a0111f4044c38ee1672c1ac6b0637d34f24907",
            "0x0eb08f6d809668a981c186beaf6110060707059576406b248e5d9cf6e78b3d3e",
            "0x07748bc6877c9b82c8b98666ee9d0626ec7f5be4205f79ee8528ef1c4a376fc7",
        ]
        .map(|n| hex_to_field(n));

        let poseidon = TestPoseidon::new(inputs);

        let out = poseidon.permute();

        assert_eq!(out, outputs);
    }
}
