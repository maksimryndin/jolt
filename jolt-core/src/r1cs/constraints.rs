use common::{constants::REGISTER_COUNT, rv_trace::CircuitFlags};
use strum::IntoEnumIterator;

use crate::{
    field::JoltField,
    jolt::{
        instruction::{
            add::ADDInstruction, mul::MULInstruction, mulhu::MULHUInstruction,
            mulu::MULUInstruction, sll::SLLInstruction, sra::SRAInstruction, srl::SRLInstruction,
            sub::SUBInstruction,
            virtual_assert_aligned_memory_access::AssertAlignedMemoryAccessInstruction,
            virtual_move::MOVEInstruction, virtual_movsign::MOVSIGNInstruction,
        },
        vm::rv_i_vm::RV_I,
    },
};

use super::{
    builder::{CombinedUniformBuilder, OffsetEqConstraint, R1CSBuilder},
    inputs::{AuxVariable, ConstraintInput, JoltR1CSInputs},
    ops::Variable,
};

pub const PC_START_ADDRESS: i64 = 0x80000000;
const PC_NOOP_SHIFT: i64 = 4;
const LOG_M: usize = 16;
const OPERAND_SIZE: usize = LOG_M / 2;

pub trait R1CSConstraints<const C: usize, F: JoltField> {
    type Inputs: ConstraintInput;
    fn construct_constraints(
        padded_trace_length: usize,
        memory_start: u64,
    ) -> CombinedUniformBuilder<C, F, Self::Inputs> {
        let mut uniform_builder = R1CSBuilder::<C, F, Self::Inputs>::new();
        Self::uniform_constraints(&mut uniform_builder, memory_start);
        let cross_step_constraints = Self::cross_step_constraints();

        CombinedUniformBuilder::construct(
            uniform_builder,
            padded_trace_length,
            cross_step_constraints,
        )
    }
    /// Constructs Jolt's uniform constraints.
    /// Uniform constraints are constraints that hold for each step of
    /// the execution trace.
    fn uniform_constraints(builder: &mut R1CSBuilder<C, F, Self::Inputs>, memory_start: u64);
    /// Construct's Jolt's cross-step constraints.
    /// Cross-step constraints are constraints whose inputs involve witness
    /// values from multiple steps of the execution trace.
    /// Currently, all of Jolt's cross-step constraints are of the form
    ///     if condition { some constraint on steps i and i+1 }
    /// This structure is captured in `OffsetEqConstraint`.
    fn cross_step_constraints() -> Vec<OffsetEqConstraint>;
}

pub struct JoltRV_IMConstraints<const WORD_SIZE: usize>;

impl<const WORD_SIZE: usize, const C: usize, F: JoltField> R1CSConstraints<C, F> for JoltRV_IMConstraints<WORD_SIZE> {
    type Inputs = JoltR1CSInputs<WORD_SIZE>;

    fn uniform_constraints(cs: &mut R1CSBuilder<C, F, Self::Inputs>, memory_start: u64) {

        for flag in RV_I::<WORD_SIZE>::iter() {
            cs.constrain_binary(JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(flag));
        }
        for flag in CircuitFlags::iter() {
            cs.constrain_binary(JoltR1CSInputs::<WORD_SIZE>::OpFlags(flag));
        }

        let flags = CircuitFlags::iter()
            .map(|flag| JoltR1CSInputs::<WORD_SIZE>::OpFlags(flag).into())
            .chain(RV_I::<WORD_SIZE>::iter().map(|flag| JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(flag).into()))
            .collect();
        cs.constrain_pack_be(flags, JoltR1CSInputs::<WORD_SIZE>::Bytecode_Bitflags, 1);

        // TODO(Maks) check for rv64
        let real_pc =
            4i64 * JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress + (PC_START_ADDRESS - PC_NOOP_SHIFT);
        let x = cs.allocate_if_else(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::LeftLookupOperand),
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::LeftOperandIsPC),
            real_pc,
            JoltR1CSInputs::<WORD_SIZE>::RS1_Read,
        );
        let y = cs.allocate_if_else(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::RightLookupOperand),
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::RightOperandIsImm),
            JoltR1CSInputs::<WORD_SIZE>::Bytecode_Imm,
            JoltR1CSInputs::<WORD_SIZE>::RS2_Read,
        );

        let is_load_or_store = JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Load)
            + JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Store);
        // TODO(Maks) check for rv 64
        let memory_start: i64 = memory_start.try_into().unwrap();
        cs.constrain_eq_conditional(
            is_load_or_store,
            JoltR1CSInputs::<WORD_SIZE>::RS1_Read + JoltR1CSInputs::<WORD_SIZE>::Bytecode_Imm,
            4 * JoltR1CSInputs::<WORD_SIZE>::RAM_Address + memory_start - 4 * REGISTER_COUNT as i64,
        );

        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Load),
            JoltR1CSInputs::<WORD_SIZE>::RAM_Read,
            JoltR1CSInputs::<WORD_SIZE>::RAM_Write,
        );
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Load),
            JoltR1CSInputs::<WORD_SIZE>::RAM_Read,
            JoltR1CSInputs::<WORD_SIZE>::RD_Write,
        );
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Store),
            JoltR1CSInputs::<WORD_SIZE>::RS2_Read,
            JoltR1CSInputs::<WORD_SIZE>::RAM_Write,
        );

        let query_chunks: Vec<Variable> = (0..C)
            .map(|i| Variable::Input(JoltR1CSInputs::<WORD_SIZE>::ChunksQuery(i).to_index::<C>()))
            .collect();
        let packed_query =
            R1CSBuilder::<C, F, JoltR1CSInputs::<WORD_SIZE>>::pack_be(query_chunks.clone(), LOG_M);

        // TODO(Maks) check for rv64
        // For the `AssertAlignedMemoryAccessInstruction` lookups, we add the `rs1` and `imm` values
        // to obtain the memory address being accessed.
        let add_operands = JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(ADDInstruction::default().into())
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(
                AssertAlignedMemoryAccessInstruction::<WORD_SIZE, 2>::default().into(),
            )
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(
                AssertAlignedMemoryAccessInstruction::<WORD_SIZE, 4>::default().into(),
            );
        cs.constrain_eq_conditional(add_operands, packed_query.clone(), x + y);
        // Converts from unsigned to twos-complement representation
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(SUBInstruction::default().into()),
            packed_query.clone(),
            x - y + (0xffffffffi64 + 1),
        );
        let is_mul = JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(MULInstruction::default().into())
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(MULUInstruction::default().into())
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(MULHUInstruction::default().into());
        let product = cs.allocate_prod(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::Product),
            JoltR1CSInputs::<WORD_SIZE>::RS1_Read,
            JoltR1CSInputs::<WORD_SIZE>::RS2_Read,
        );
        cs.constrain_eq_conditional(is_mul, packed_query.clone(), product);
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(MOVSIGNInstruction::default().into())
                + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(MOVEInstruction::default().into()),
            packed_query.clone(),
            x,
        );

        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Assert),
            JoltR1CSInputs::<WORD_SIZE>::LookupOutput,
            1,
        );

        let x_chunks: Vec<Variable> = (0..C)
            .map(|i| Variable::Input(JoltR1CSInputs::<WORD_SIZE>::ChunksX(i).to_index::<C>()))
            .collect();
        let y_chunks: Vec<Variable> = (0..C)
            .map(|i| Variable::Input(JoltR1CSInputs::<WORD_SIZE>::ChunksY(i).to_index::<C>()))
            .collect();
        let x_concat = R1CSBuilder::<C, F, JoltR1CSInputs::<WORD_SIZE>>::pack_be(x_chunks.clone(), OPERAND_SIZE);
        let y_concat = R1CSBuilder::<C, F, JoltR1CSInputs::<WORD_SIZE>>::pack_be(y_chunks.clone(), OPERAND_SIZE);
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::ConcatLookupQueryChunks),
            x_concat,
            x,
        );
        cs.constrain_eq_conditional(
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::ConcatLookupQueryChunks),
            y_concat,
            y,
        );

        // if is_shift ? chunks_query[i] == zip(chunks_x[i], chunks_y[C-1]) : chunks_query[i] == zip(chunks_x[i], chunks_y[i])
        let is_shift = JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(SLLInstruction::default().into())
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(SRLInstruction::default().into())
            + JoltR1CSInputs::<WORD_SIZE>::InstructionFlags(SRAInstruction::default().into());
        for i in 0..C {
            let relevant_chunk_y = cs.allocate_if_else(
                JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::RelevantYChunk(i)),
                is_shift.clone(),
                y_chunks[C - 1],
                y_chunks[i],
            );
            cs.constrain_eq_conditional(
                JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::ConcatLookupQueryChunks),
                query_chunks[i],
                x_chunks[i] * (1i64 << 8) + relevant_chunk_y,
            );
        }

        // if (rd != 0 && update_rd_with_lookup_output == 1) constrain(rd_val == LookupOutput)
        let rd_nonzero_and_lookup_to_rd = cs.allocate_prod(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::WriteLookupOutputToRD),
            JoltR1CSInputs::<WORD_SIZE>::Bytecode_RD,
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::WriteLookupOutputToRD),
        );
        cs.constrain_eq_conditional(
            rd_nonzero_and_lookup_to_rd,
            JoltR1CSInputs::<WORD_SIZE>::RD_Write,
            JoltR1CSInputs::<WORD_SIZE>::LookupOutput,
        );
        // if (rd != 0 && is_jump_instr == 1) constrain(rd_val == 4 * PC)
        let rd_nonzero_and_jmp = cs.allocate_prod(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::WritePCtoRD),
            JoltR1CSInputs::<WORD_SIZE>::Bytecode_RD,
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Jump),
        );
        cs.constrain_eq_conditional(
            rd_nonzero_and_jmp,
            4 * JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress + PC_START_ADDRESS,
            JoltR1CSInputs::<WORD_SIZE>::RD_Write,
        );

        let next_pc_jump = cs.allocate_if_else(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::NextPCJump),
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Jump),
            JoltR1CSInputs::<WORD_SIZE>::LookupOutput + 4,
            4 * JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress + PC_START_ADDRESS + 4
                - 4 * JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::DoNotUpdatePC),
        );

        let should_branch = cs.allocate_prod(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::ShouldBranch),
            JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Branch),
            JoltR1CSInputs::<WORD_SIZE>::LookupOutput,
        );
        let _next_pc = cs.allocate_if_else(
            JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::NextPC),
            should_branch,
            4 * JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress
                + PC_START_ADDRESS
                + JoltR1CSInputs::<WORD_SIZE>::Bytecode_Imm,
            next_pc_jump,
        );
    }

    fn cross_step_constraints() -> Vec<OffsetEqConstraint> {
        // If the next instruction's ELF address is not zero (i.e. it's
        // not padding), then check the PC update.
        let pc_constraint = OffsetEqConstraint::new(
            (JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress, true),
            (JoltR1CSInputs::<WORD_SIZE>::Aux(AuxVariable::NextPC), false),
            (
                4 * JoltR1CSInputs::<WORD_SIZE>::Bytecode_ELFAddress + PC_START_ADDRESS,
                true,
            ),
        );

        // If the current instruction is virtual, check that the next instruction
        // in the trace is the next instruction in bytecode. Virtual sequences
        // do not involve jumps or branches, so this should always hold,
        // EXCEPT if we encounter a virtual instruction followed by a padding
        // instruction. But that should never happen because the execution
        // trace should always end with some return handling, which shouldn't involve
        // any virtual sequences.
        let virtual_sequence_constraint = OffsetEqConstraint::new(
            (JoltR1CSInputs::<WORD_SIZE>::OpFlags(CircuitFlags::Virtual), false),
            (JoltR1CSInputs::<WORD_SIZE>::Bytecode_A, true),
            (JoltR1CSInputs::<WORD_SIZE>::Bytecode_A + 1, false),
        );

        vec![pc_constraint, virtual_sequence_constraint]
    }
}
