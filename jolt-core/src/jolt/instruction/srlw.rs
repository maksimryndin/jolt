use crate::field::JoltField;
use rand::prelude::StdRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::{JoltInstruction, SubtableIndices};
use crate::jolt::subtable::{srl::SrlSubtable, LassoSubtable};
use crate::utils::instruction_utils::{assert_valid_parameters, chunk_and_concatenate_for_shift};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct SRLWInstruction(pub u64, pub u64);

impl SRLWInstruction {
    const WORD_SIZE: usize = 64;
}

impl JoltInstruction for SRLWInstruction {
    fn operands(&self) -> (u64, u64) {
        (self.0, self.1)
    }

    fn combine_lookups<F: JoltField>(&self, vals: &[F], C: usize, _: usize) -> F {
        assert!(C <= 10);
        assert!(vals.len() == C);
        vals.iter().sum()
    }

    fn g_poly_degree(&self, _: usize) -> usize {
        1
    }

    fn subtables<F: JoltField>(
        &self,
        C: usize,
        _: usize,
    ) -> Vec<(Box<dyn LassoSubtable<F>>, SubtableIndices)> {
        // We have to pre-define subtables in this way because `CHUNK_INDEX` needs to be a constant,
        // i.e. known at compile time (so we cannot do a `map` over the range of `C`,
        // which only happens at runtime).
        let mut subtables: Vec<Box<dyn LassoSubtable<F>>> = vec![
            Box::new(SrlSubtable::<F, 0, 32>::new()),
            Box::new(SrlSubtable::<F, 1, 32>::new()),
            Box::new(SrlSubtable::<F, 2, 32>::new()),
            Box::new(SrlSubtable::<F, 3, 32>::new()),

        ];
        subtables.truncate(C);
        subtables.reverse();

        let indices = (0..C).map(SubtableIndices::from);
        subtables.into_iter().zip(indices).collect()
    }

    fn to_indices(&self, C: usize, log_M: usize) -> Vec<usize> {
        assert_valid_parameters(Self::WORD_SIZE, C, log_M);
        chunk_and_concatenate_for_shift(self.0, self.1, C, log_M)
    }

    fn lookup_entry(&self) -> u64 {
        let x = self.0 as u32;
        let y = (self.1 % 32) as u32;
        (x.wrapping_shr(y)).into()
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

    use super::SRLWInstruction;

    //TODO(Maks) update test
    #[test]
    #[ignore]
    fn srl_instruction_64_e2e() {
        let mut rng = test_rng();
        const C: usize = 8;
        const M: usize = 1 << 16;
        const WORD_SIZE: usize = 64;

        for _ in 0..256 {
            let (x, y) = (rng.next_u64(), rng.next_u64());
            let instruction = SRLInstruction::<WORD_SIZE>(x, y);
            jolt_instruction_test!(instruction);
        }

        let u64_max: u64 = u64::MAX;
        let instructions = vec![
            SRLInstruction::<64>(100, 0),
            SRLInstruction::<64>(0, 2),
            SRLInstruction::<64>(1, 2),
            SRLInstruction::<64>(0, 64),
            SRLInstruction::<64>(u64_max, 0),
            SRLInstruction::<64>(u64_max, 63),
            SRLInstruction::<64>(u64_max, 1 << 8),
            SRLInstruction::<64>(1 << 32, 1 << 16),
            SRLInstruction::<64>(1 << 63, 1),
            SRLInstruction::<64>((1 << 63) - 1, 1),
        ];

        for instruction in instructions {
            jolt_instruction_test!(instruction);
        }
    }
}
