use crate::jolt::instruction::and::ANDInstruction;
use crate::jolt::instruction::beq::BEQInstruction;
use crate::jolt::instruction::bge::BGEInstruction;
use crate::jolt::instruction::bgeu::BGEUInstruction;
use crate::jolt::instruction::bne::BNEInstruction;
use crate::jolt::instruction::mul::MULInstruction;
use crate::jolt::instruction::mulhu::MULHUInstruction;
use crate::jolt::instruction::mulu::MULUInstruction;
use crate::jolt::instruction::or::ORInstruction;
use crate::jolt::instruction::sll::SLLInstruction;
use crate::jolt::instruction::slt::SLTInstruction;
use crate::jolt::instruction::sltu::SLTUInstruction;
use crate::jolt::instruction::sra::SRAInstruction;
use crate::jolt::instruction::srl::SRLInstruction;
use crate::jolt::instruction::sub::SUBInstruction;
use crate::jolt::instruction::virtual_advice::ADVICEInstruction;
use crate::jolt::instruction::virtual_assert_aligned_memory_access::AssertAlignedMemoryAccessInstruction;
use crate::jolt::instruction::virtual_assert_lte::ASSERTLTEInstruction;
use crate::jolt::instruction::virtual_assert_valid_div0::AssertValidDiv0Instruction;
use crate::jolt::instruction::virtual_assert_valid_signed_remainder::AssertValidSignedRemainderInstruction;
use crate::jolt::instruction::virtual_assert_valid_unsigned_remainder::AssertValidUnsignedRemainderInstruction;
use crate::jolt::instruction::virtual_move::MOVEInstruction;
use crate::jolt::instruction::xor::XORInstruction;
use crate::jolt::instruction::{add::ADDInstruction, virtual_movsign::MOVSIGNInstruction};
use crate::jolt::vm::rv_i_vm::RV_I;
use common::rv_trace::{ELFInstruction, RVTraceRow, RV_IM};

impl<const WORD_SIZE: usize> TryFrom<&ELFInstruction<WORD_SIZE>> for RV_I<WORD_SIZE> {
    type Error = &'static str;

    #[rustfmt::skip] // keep matches pretty
    fn try_from(instruction: &ELFInstruction<WORD_SIZE>) -> Result<Self, Self::Error> {
        // TODO(Maks) check for rv64
        use RV_IM::*;
        match instruction.opcode {
            ADD  => Ok(ADDInstruction::default().into()),
            SUB  => Ok(SUBInstruction::default().into()),
            XOR  => Ok(XORInstruction::default().into()),
            OR   => Ok(ORInstruction::default().into()),
            AND  => Ok(ANDInstruction::default().into()),
            SLL  => Ok(SLLInstruction::default().into()),
            SRL  => Ok(SRLInstruction::default().into()),
            SRA  => Ok(SRAInstruction::default().into()),
            SLT  => Ok(SLTInstruction::default().into()),
            SLTU => Ok(SLTUInstruction::default().into()),

            ADDI  => Ok(ADDInstruction::default().into()),
            XORI  => Ok(XORInstruction::default().into()),
            ORI   => Ok(ORInstruction::default().into()),
            ANDI  => Ok(ANDInstruction::default().into()),
            SLLI  => Ok(SLLInstruction::default().into()),
            SRLI  => Ok(SRLInstruction::default().into()),
            SRAI  => Ok(SRAInstruction::default().into()),
            SLTI  => Ok(SLTInstruction::default().into()),
            SLTIU => Ok(SLTUInstruction::default().into()),

            BEQ  => Ok(BEQInstruction::default().into()),
            BNE  => Ok(BNEInstruction::default().into()),
            BLT  => Ok(SLTInstruction::default().into()),
            BLTU => Ok(SLTUInstruction::default().into()),
            BGE  => Ok(BGEInstruction::default().into()),
            BGEU => Ok(BGEUInstruction::default().into()),

            JAL   => Ok(ADDInstruction::default().into()),
            JALR  => Ok(ADDInstruction::default().into()),
            AUIPC => Ok(ADDInstruction::default().into()),

            MUL => Ok(MULInstruction::default().into()),
            MULU => Ok(MULUInstruction::default().into()),
            MULHU => Ok(MULHUInstruction::default().into()),

            VIRTUAL_ADVICE => Ok(ADVICEInstruction::default().into()),
            VIRTUAL_MOVE => Ok(MOVEInstruction::default().into()),
            VIRTUAL_MOVSIGN => Ok(MOVSIGNInstruction::default().into()),
            VIRTUAL_ASSERT_EQ => Ok(BEQInstruction::default().into()),
            VIRTUAL_ASSERT_LTE => Ok(ASSERTLTEInstruction::default().into()),
            VIRTUAL_ASSERT_VALID_UNSIGNED_REMAINDER => Ok(AssertValidUnsignedRemainderInstruction::default().into()),
            VIRTUAL_ASSERT_VALID_SIGNED_REMAINDER => Ok(AssertValidSignedRemainderInstruction::default().into()),
            VIRTUAL_ASSERT_VALID_DIV0 => Ok(AssertValidDiv0Instruction::default().into()),
            VIRTUAL_ASSERT_HALFWORD_ALIGNMENT => Ok(AssertAlignedMemoryAccessInstruction::<WORD_SIZE, 2>::default().into()),
            // TODO(Maks) add 64b instructions
            _ if WORD_SIZE == 64 => Err("No corresponding RV64I instruction"),
            _ => Err("No corresponding RV32I instruction")
        }
    }
}

impl<const WORD_SIZE: usize> TryFrom<&RVTraceRow<WORD_SIZE>> for RV_I<WORD_SIZE> {
    type Error = &'static str;

    #[rustfmt::skip] // keep matches pretty
    fn try_from(row: &RVTraceRow<WORD_SIZE>) -> Result<Self, Self::Error> {
        // TODO(Maks) check for rv64
        use RV_IM::*;
        match row.instruction.opcode {
            ADD => Ok(ADDInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SUB => Ok(SUBInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            XOR => Ok(XORInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            OR  => Ok(ORInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            AND => Ok(ANDInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SLL => Ok(SLLInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SRL => Ok(SRLInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SRA => Ok(SRAInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SLT  => Ok(SLTInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            SLTU => Ok(SLTUInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),

            ADDI  => Ok(ADDInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            XORI  => Ok(XORInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            ORI   => Ok(ORInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            ANDI  => Ok(ANDInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            SLLI  => Ok(SLLInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            SRLI  => Ok(SRLInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            SRAI  => Ok(SRAInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            SLTI  => Ok(SLTInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            SLTIU => Ok(SLTUInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),

            BEQ  => Ok(BEQInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            BNE  => Ok(BNEInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            BLT  => Ok(SLTInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            BLTU => Ok(SLTUInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            BGE  => Ok(BGEInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            BGEU => Ok(BGEUInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),

            JAL  => Ok(ADDInstruction(row.instruction.address, row.imm()).into()),
            JALR => Ok(ADDInstruction(row.register_state.rs1_val.unwrap(), row.imm()).into()),
            AUIPC => Ok(ADDInstruction(row.instruction.address, row.imm()).into()),

            MUL => Ok(MULInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            MULU => Ok(MULUInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            MULHU => Ok(MULHUInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),

            VIRTUAL_ADVICE => Ok(ADVICEInstruction(row.advice_value.unwrap()).into()),
            VIRTUAL_MOVE => Ok(MOVEInstruction(row.register_state.rs1_val.unwrap()).into()),
            VIRTUAL_MOVSIGN => Ok(MOVSIGNInstruction(row.register_state.rs1_val.unwrap()).into()),
            VIRTUAL_ASSERT_EQ => Ok(BEQInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            VIRTUAL_ASSERT_LTE => Ok(ASSERTLTEInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            VIRTUAL_ASSERT_VALID_UNSIGNED_REMAINDER => Ok(AssertValidUnsignedRemainderInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            VIRTUAL_ASSERT_VALID_SIGNED_REMAINDER => Ok(AssertValidSignedRemainderInstruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            VIRTUAL_ASSERT_VALID_DIV0 => Ok(AssertValidDiv0Instruction(row.register_state.rs1_val.unwrap(), row.register_state.rs2_val.unwrap()).into()),
            VIRTUAL_ASSERT_HALFWORD_ALIGNMENT => Ok(AssertAlignedMemoryAccessInstruction::<WORD_SIZE, 2>(row.register_state.rs1_val.unwrap(), row.imm()).into()),

            // TODO(Maks) add 64b instructions
            _ if WORD_SIZE == 64 => Err("No corresponding RV64I instruction"),
            _ => Err("No corresponding RV32I instruction")
        }
    }
}
