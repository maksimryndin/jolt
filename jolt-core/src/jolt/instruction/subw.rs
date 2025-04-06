use crate::field::JoltField;
use ark_std::log2;
use rand::prelude::StdRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::{JoltInstruction, SubtableIndices};
use crate::jolt::subtable::{identity::IdentitySubtable, LassoSubtable};
use crate::utils::instruction_utils::{
    add_and_chunk_operands, assert_valid_parameters, concatenate_lookups,
};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct SUBWInstruction(pub u64, pub u64);

impl SUBWInstruction {
    const WORD_SIZE: usize = 64;
}

impl JoltInstruction for SUBWInstruction {
    fn operands(&self) -> (u64, u64) {
        (self.0, self.1)
    }

    fn combine_lookups<F: JoltField>(&self, vals: &[F], C: usize, M: usize) -> F {
        assert!(vals.len() == C / 2);
        // The output is Identity of lower chunks
        concatenate_lookups(vals, C / 2, log2(M) as usize)
    }

    fn g_poly_degree(&self, _: usize) -> usize {
        1
    }

    fn subtables<F: JoltField>(
        &self,
        C: usize,
        M: usize,
    ) -> Vec<(Box<dyn LassoSubtable<F>>, SubtableIndices)> {
        let msb_chunk_index = C - (Self::WORD_SIZE / log2(M) as usize) - 1;
        vec![(
            Box::new(IdentitySubtable::new()),
            SubtableIndices::from(msb_chunk_index + 1..C),
        )]
    }

    fn to_indices(&self, C: usize, log_M: usize) -> Vec<usize> {
        assert_valid_parameters(Self::WORD_SIZE, C, log_M);
        add_and_chunk_operands(
            self.0 as u128,
            (1u128 << Self::WORD_SIZE) - self.1 as u128,
            C,
            log_M,
        )
    }

    fn lookup_entry(&self) -> u64 {
        (self.0 as u32).overflowing_sub(self.1 as u32).0.into()
    }

    fn random(&self, rng: &mut StdRng) -> Self {
        Self(rng.next_u32() as u64, rng.next_u32() as u64)
    }
}

#[cfg(test)]
mod test {
    use ark_bn254::Fr;
    use ark_std::test_rng;
    use rand_chacha::rand_core::RngCore;

    use crate::{jolt::instruction::JoltInstruction, jolt_instruction_test};

    use super::SUBWInstruction;

    //TODO(Maks) update test
    #[test]
    fn sub_instruction_64_e2e() {
        let mut rng = test_rng();
        const C: usize = 8;
        const M: usize = 1 << 16;
        const WORD_SIZE: usize = 64;

        // Random
        for _ in 0..256 {
            let (x, y) = (rng.next_u64(), rng.next_u64());
            let instruction = SUBInstruction::<WORD_SIZE>(x, y);
            jolt_instruction_test!(instruction);
        }

        // Edge cases
        let u64_max: u64 = u64::MAX;
        let instructions = vec![
            SUBInstruction::<WORD_SIZE>(100, 0),
            SUBInstruction::<WORD_SIZE>(0, 100),
            SUBInstruction::<WORD_SIZE>(1, 0),
            SUBInstruction::<WORD_SIZE>(0, u64_max),
            SUBInstruction::<WORD_SIZE>(u64_max, 0),
            SUBInstruction::<WORD_SIZE>(u64_max, u64_max),
            SUBInstruction::<WORD_SIZE>(u64_max, 1 << 8),
            SUBInstruction::<WORD_SIZE>(1 << 8, u64_max),
            SUBInstruction::<WORD_SIZE>(u64_max, 1 << 32),
            SUBInstruction::<WORD_SIZE>(1 << 32, u64_max),
        ];
        for instruction in instructions {
            jolt_instruction_test!(instruction);
        }
    }
}
