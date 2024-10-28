use super::{Register, RegisterPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    Always,
    Z, NZ,
    C, NC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    LdR8FromHl(Register),
    LdR8ToHl(Register),
    LdR8Imm(Register, u8),
    LdR8R8(Register, Register),
    LdR16Imm(RegisterPair, u16),
    Jr(Condition, i8),
    IncR16(RegisterPair),
    LdToHlAdd,
    LdToHlSub,
    LdFromHlAdd,
    LdFromHlA,
    LdHlImm(u8),
    Jp(u16),
}

impl From<Instruction> for Vec<u8> {
    fn from(value: Instruction) -> Self {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        match value {
            LdR8FromHl(A) => vec![0x7e],
            LdR16Imm(r16, imm) => {
                let imm = imm.to_le_bytes();
                let mut opcode = match r16 {
                    HL => vec![0x21],
                    _ => unimplemented!()
                };
                opcode.extend(imm);
                opcode
            }
            LdR8Imm(A, imm) => vec![0x3e, imm],
            LdHlImm(imm) => vec![0x36, imm],
            LdToHlAdd => vec![0x22],
            LdToHlSub => vec![0x32],
            Jp(imm) => {
                let imm = imm.to_le_bytes();
                vec![0xc3, imm[0], imm[1]]
            },
            Jr(Condition::Always, imm) => vec![0x18, imm as u8],
            _ => unimplemented!()
        }.to_vec()
    }
}