use crate::codegen::Id;

use super::{CpuFlag, GpRegister, IndirectPair, RegisterPair, StackPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    // Native conditions
    Flag(CpuFlag),
    Always,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bit {
    _0, _1, _2, _3,
    _4, _5, _6, _7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    LdR16Imm(RegisterPair, u16),
    LdAFromR16(IndirectPair),
    IncR16(RegisterPair),
    IncR8(GpRegister),
    DecR8(GpRegister),
    LdR8Imm(GpRegister, u8),
    DecR16(RegisterPair),
    LdAToR16(IndirectPair),
    Jr(Condition, i8),
    LdR8FromR8(GpRegister, GpRegister),
    Pop(StackPair),
    Jp(Condition, u16),
    Push(StackPair),
    Prefixed(PrefixInstruction),
    LdhFromA(u8),
    LdIndFromA(u16),
    LdhToA(u8),
    LdAFromInd(u16),
    /// pretend this is an actual instruction (won't be emitted into the rom)
    Label(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixInstruction {
    Bit(Bit, GpRegister),
    Res(Bit, GpRegister),
    Set(Bit, GpRegister),
}

impl Instruction {
    pub const PREFIX: u8 = 0xcb;

    pub fn len(&self) -> usize {
        use Instruction::*;

        match self {
            LdR16Imm(_, _) => 3,
            LdAFromR16(_) => 1,
            IncR16(_) => 1,
            IncR8(_) => 1,
            DecR8(_) => 1,
            LdR8Imm(_, _) => 2,
            DecR16(_) => 1,
            LdAToR16(_) => 1,
            Jr(_, _) => 2,
            LdR8FromR8(_, _) => 1,
            Pop(_) => 1,
            Jp(_, _) => 3,
            Push(_) => 1,
            Prefixed(_) => 2,
            LdhFromA(_) => 2,
            LdIndFromA(_) => 3,
            LdhToA(_) => 2,
            LdAFromInd(_) => 3,
            Label(_) => 0,
        }
    }

    fn base(&self) -> u8 {
        match self {
            Self::LdR16Imm(_, _) => 0x01,
            Self::LdAFromR16(_) => 0x02,
            Self::IncR16(_) => 0x03,
            Self::IncR8(_) => 0x04,
            Self::DecR8(_) => 0x05,
            Self::LdR8Imm(_, _) => 0x06,
            Self::DecR16(_) => 0x08,
            Self::LdAToR16(_) => 0x0a,
            Self::Jr(Condition::Always, _) => 0x18,
            Self::Jr(Condition::Flag(_), _) => 0x20,
            Self::LdR8FromR8(_, _) => 0x40,
            Self::Pop(_) => 0xc1,
            Self::Jp(Condition::Flag(_), _) => 0xc2,
            Self::Jp(Condition::Always, _) => 0xc3,
            Self::Push(_) => 0xc5,
            Self::Prefixed(_) => 0xcb,
            Self::LdhFromA(_) => 0xe0,
            Self::LdIndFromA(_) => 0xea,
            Self::LdhToA(_) => 0xf0,
            Self::LdAFromInd(_) => 0xfa,
            Self::Label(_) => 0xd3, // illegal opcode since these shouldnt be emitted
        }
    }
}

impl PrefixInstruction {
    fn base(&self) -> u8 {
        match self {
            Self::Bit(_, _) => 0x40,
            Self::Res(_, _) => 0x80,
            Self::Set(_, _) => 0xc0,
        }
    }
}

impl From<Instruction> for Vec<u8> {
    fn from(value: Instruction) -> Self {
        use Instruction::*;
        let mut out: Vec<u8> = Vec::with_capacity(3);
        out.push(value.base());

        match value {
            LdR8FromR8(GpRegister::IndHL, GpRegister::IndHL) => todo!("idk how to handle this"),
            LdR8FromR8(to, from) => out[0] += (to as u8 * 0x08) + from as u8,
            LdR16Imm(r16, imm) => {
                out[0] += r16 as u8 * 0x10;
                out.extend(imm.to_le_bytes());
            }
            LdAFromR16(r16) => out[0] += r16 as u8 * 0x10,
            LdR8Imm(r8, imm) => {
                out[0] += r8 as u8 * 0x08;
                out.push(imm);
            }
            LdhFromA(imm)
            | LdhToA(imm) => out.push(imm),
            IncR8(r8)
            | DecR8(r8) => out[0] += r8 as u8 * 0x08,
            IncR16(r16)
            | DecR16(r16) => out[0] += r16 as u8 * 0x10,
            v @ Jp(condition, _)
            | v @ Jr(condition, _) => {
                out[0] += match condition {
                    Condition::Always => 0,
                    Condition::Flag(flag) => flag as u8 * 0x08,
                };

                match v {
                    Jp(_, imm) => out.extend(imm.to_le_bytes()),
                    Jr(_, imm) => out.push(imm as u8),
                    _ => unreachable!("Filtered down to just Jp|Jr in the outer match")
                }
            },
            Push(r16)
            | Pop(r16) => out[0] += r16 as u8 * 0x10,
            Prefixed(instruction) => out.push(instruction.into()),
            Label(_) => {},
            e => unimplemented!("There is no {:?}", e)
        };

        out
    }
}

impl From<PrefixInstruction> for u8 {
    fn from(value: PrefixInstruction) -> Self {
        use PrefixInstruction as Pre;

        match value {
            Pre::Bit(bit, reg) | Pre::Res(bit, reg) | Pre::Set(bit, reg) => {
                let base = value.base();
                let reg_offset = reg as u8;
                let bit_offset = bit as u8 * 0x08;
                base + reg_offset + bit_offset
            },
        }
    }
}