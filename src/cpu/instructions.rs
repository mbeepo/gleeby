use crate::codegen::Id;

use super::{Register, RegisterPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    // Native conditions
    Always,
    Z, NZ,
    C, NC
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bit {
    _0, _1, _2, _3,
    _4, _5, _6, _7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    LdR8FromHl(Register),
    LdR8ToHl(Register),
    LdR8Imm(Register, u8),
    LdR8FromR8(Register, Register),
    LdR16Imm(RegisterPair, u16),
    LdAFromR16(RegisterPair),
    LdAToR16(RegisterPair),
    Jr(Condition, i8),
    IncR8(Register),
    DecR8(Register),
    IncR16(RegisterPair),
    DecR16(RegisterPair),
    LdToHlInc,
    LdToHlDec,
    LdFromHlInc,
    LdFromHlDec,
    LdHlImm(u8),
    LdhFromA(u8),
    LdhToA(u8),
    Jp(u16),
    Bit(Register, Bit),
    Res(Register, Bit),
    Set(Register, Bit),
    /// pretend this is an actual instruction
    /// 
    /// it wont be included in the rom
    Label(Id),
}

impl Instruction {
    pub const PREFIX: u8 = 0xcb;

    pub fn len(&self) -> usize {
        use Instruction::*;

        match self {
            LdR8FromHl(_) => 1,
            LdR8ToHl(_) => 1,
            LdR8Imm(_, _) => 2,
            LdR8FromR8(_, _) => 1,
            LdR16Imm(_, _) => 3,
            LdAFromR16(_) => 1,
            LdAToR16(_) => 1,
            Jr(_, _) => 2,
            IncR8(_) => 1,
            DecR8(_) => 1,
            IncR16(_) => 1,
            DecR16(_) => 1,
            LdToHlInc => 1,
            LdToHlDec => 1,
            LdFromHlInc => 1,
            LdFromHlDec => 1,
            LdHlImm(_) => 2,
            LdhFromA(_) => 2,
            LdhToA(_) => 2,
            Jp(_) => 3,
            Res(_, _) => 2,
            Label(_) => 0,
            _ => todo!()
        }
    }

    fn gen_prefixed(&self) -> Result<Vec<u8>, ()> {
        use Register::*;
        let (reg, bit, base) = match self {
            Instruction::Bit(reg, bit) => (reg, bit, 0x40),
            Instruction::Res(reg, bit) => (reg, bit, 0x80),
            Instruction::Set(reg, bit) => (reg, bit, 0xc0),
            _ => return Err(()),
        };

        let low_nibble: u8 = match reg {
            A => 0x07,
            B => 0x00,
            C => 0x01,
            D => 0x02,
            E => 0x03,
            H => 0x04,
            L => 0x05,
            IndHL => 0x06,
        };

        let high_nibble: u8 = match bit {
            Bit::_0 => 0x00,
            Bit::_1 => 0x08,
            Bit::_2 => 0x10,
            Bit::_3 => 0x18,
            Bit::_4 => 0x20,
            Bit::_5 => 0x28,
            Bit::_6 => 0x30,
            Bit::_7 => 0x38,
        };

        let byte = base + low_nibble + high_nibble;
        Ok(vec![Instruction::PREFIX, byte])
    }
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
                    BC => vec![0x01],
                    DE => vec![0x11],
                    HL => vec![0x21],
                    SP => vec![0x31],
                };
                opcode.extend(imm);
                opcode
            }
            LdAFromR16(BC) => vec![0x0a],
            LdAFromR16(DE) => vec![0x1a],
            LdAFromR16(HL) => vec![0x7e],
            LdR8Imm(A, imm) => vec![0x3e, imm],
            LdR8Imm(B, imm) => vec![0x06, imm],
            LdR8Imm(C, imm) => vec![0x0e, imm],
            LdR8Imm(D, imm) => vec![0x16, imm],
            LdR8Imm(E, imm) => vec![0x1e, imm],
            LdR8Imm(H, imm) => vec![0x26, imm],
            LdR8Imm(L, imm) => vec![0x2e, imm],
            LdHlImm(imm) => vec![0x36, imm],
            LdhFromA(imm) => vec![0xe0, imm],
            LdhToA(imm) => vec![0xf0, imm],
            IncR8(A) => vec![0x3c],
            IncR8(B) => vec![0x04],
            IncR8(C) => vec![0x0c],
            IncR8(D) => vec![0x14],
            IncR8(E) => vec![0x1c],
            IncR8(H) => vec![0x24],
            IncR8(L) => vec![0x2c],
            DecR8(A) => vec![0x3d],
            DecR8(B) => vec![0x05],
            DecR8(C) => vec![0x0d],
            DecR8(D) => vec![0x15],
            DecR8(E) => vec![0x1d],
            DecR8(H) => vec![0x25],
            DecR8(L) => vec![0x2d],
            IncR16(BC) => vec![0x03],
            IncR16(DE) => vec![0x13],
            IncR16(HL) => vec![0x23],
            IncR16(SP) => vec![0x33],
            DecR16(BC) => vec![0x0b],
            DecR16(DE) => vec![0x1b],
            DecR16(HL) => vec![0x2b],
            DecR16(SP) => vec![0x3b],
            LdToHlInc => vec![0x22],
            LdToHlDec => vec![0x32],
            LdFromHlInc => vec![0x2a],
            LdFromHlDec => vec![0x3a],
            Jp(imm) => {
                let imm = imm.to_le_bytes();
                vec![0xc3, imm[0], imm[1]]
            },
            Jr(Condition::Always, imm) => vec![0x18, imm as u8],
            Jr(Condition::NZ, imm) => vec![0x20, imm as u8],
            Jr(Condition::Z, imm) => vec![0x28, imm as u8],
            Jr(Condition::NC, imm) => vec![0x30, imm as u8],
            Jr(Condition::C, imm) => vec![0x38, imm as u8],
            s @ Bit(_, _) | s @ Res(_, _) | s @ Set(_, _) => s.gen_prefixed().unwrap(),
            Label(_) => vec![],
            e => unimplemented!("There is no {:?}", e)
        }.to_vec()
    }
}