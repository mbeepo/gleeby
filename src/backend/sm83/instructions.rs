use super::registers::{CpuFlag, GpRegister, IndirectPair, RegisterPair, StackPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    // Native conditions
    Flag(CpuFlag),
    Always,
}

impl From<CpuFlag> for Condition {
    fn from(value: CpuFlag) -> Self {
        Self::Flag(value)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bit {
    _0, _1, _2, _3,
    _4, _5, _6, _7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// 0x00
    Nop,
    /// - **BC:** 0x01
    /// - **DE:** 0x11
    /// - **HL:** 0x21
    /// - **SP:** 0x31
    LdR16Imm(RegisterPair, u16),
    /// - **BC:** 0x02
    /// - **DE:** 0x12
    /// - **HL:** 0x22
    /// - **SP:** 0x32
    LdAToR16(IndirectPair),
    /// - **BC:** 0x03
    /// - **DE:** 0x13
    /// - **HL:** 0x23
    /// - **SP:** 0x33
    IncR16(RegisterPair),
    /// - **B:** 0x04
    /// - **C:** 0x0c
    /// - **D:** 0x14
    /// - **E:** 0x1c
    /// - **H:** 0x24
    /// - **L:** 0x2c
    /// - **(HL):** 0x34
    /// - **A:** 0x3c
    IncR8(GpRegister),
    /// - **B:** 0x05
    /// - **C:** 0x0d
    /// - **D:** 0x15
    /// - **E:** 0x1d
    /// - **H:** 0x25
    /// - **L:** 0x2d
    /// - **(HL):** 0x35
    /// - **A:** 0x3d
    DecR8(GpRegister),
    /// - **B:** 0x06
    /// - **C:** 0x0e
    /// - **D:** 0x16
    /// - **E:** 0x1e
    /// - **H:** 0x26
    /// - **L:** 0x2e
    /// - **(HL):** 0x36
    /// - **A:** 0x3e
    LdR8Imm(GpRegister, u8),
    /// - **(BC):** 0x0a
    /// - **(DE):** 0x1a
    /// - **(HL+):** 0x2a
    /// - **(HL-):** 0x3a
    LdAFromR16(IndirectPair),
    /// - **BC:** 0x0b
    /// - **DE:** 0x1b
    /// - **HL:** 0x2b
    /// - **SP:** 0x3b
    DecR16(RegisterPair),
    /// - **Always:** 0x18
    /// - **Not Zero:** 0x20
    /// - **Zero:** 0x28
    /// - **Not Carry:** 0x30
    /// - **Carry:** 0x38
    Jr(Condition, i8),
    /// 0x40 - 0x7f
    LdR8FromR8(GpRegister, GpRegister),
    /// 0xb8 - 0xbf
    Cp(GpRegister),
    /// - **BC:** 0xc1
    /// - **DE:** 0xd1
    /// - **HL:** 0xe1
    /// - **SP:** 0xf1
    Pop(StackPair),
    /// - **Always:** 0xc3
    /// - **Not Zero:** 0xc2
    /// - **Zero:** 0xca
    /// - **Not Carry:** 0xd2
    /// - **Carry:** 0xda
    Jp(Condition, u16),
    /// - **BC:** 0xc5
    /// - **DE:** 0xd5
    /// - **HL:** 0xe5
    /// - **SP:** 0xf5
    Push(StackPair),
    /// 0xcb
    Prefixed(PrefixInstruction),
    /// 0xe0
    LdhFromA(u8),
    /// 0xe2
    LdhPlusCFromA,
    /// 0xea
    LdAToInd(u16),
    /// 0xf0
    LdhToA(u8),
    /// 0xf2
    LdhPlusCToA,
    /// 0xfa
    LdAFromInd(u16),
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
            Nop => 1,
            LdR16Imm(_, _) => 3,
            LdAToR16(_) => 1,
            IncR16(_) => 1,
            IncR8(_) => 1,
            DecR8(_) => 1,
            LdR8Imm(_, _) => 2,
            LdAFromR16(_) => 1,
            DecR16(_) => 1,
            Jr(_, _) => 2,
            LdR8FromR8(_, _) => 1,
            Cp(_) => 1,
            Pop(_) => 1,
            Jp(_, _) => 3,
            Push(_) => 1,
            Prefixed(_) => 2,
            LdhFromA(_) => 2,
            LdhPlusCFromA => 1,
            LdAToInd(_) => 3,
            LdhToA(_) => 2,
            LdhPlusCToA => 1,
            LdAFromInd(_) => 3,
        }
    }

    fn base(&self) -> u8 {
        use Instruction::*;

        match self {
            Nop => 0x00,
            LdR16Imm(_, _) => 0x01,
            LdAToR16(_) => 0x02,
            IncR16(_) => 0x03,
            IncR8(_) => 0x04,
            DecR8(_) => 0x05,
            LdR8Imm(_, _) => 0x06,
            LdAFromR16(_) => 0x0a,
            DecR16(_) => 0x0b,
            Jr(Condition::Always, _) => 0x18,
            Jr(Condition::Flag(_), _) => 0x20,
            LdR8FromR8(_, _) => 0x40,
            Cp(_) => 0xb8,
            Pop(_) => 0xc1,
            Jp(Condition::Flag(_), _) => 0xc2,
            Jp(Condition::Always, _) => 0xc3,
            Push(_) => 0xc5,
            Prefixed(_) => 0xcb,
            LdhFromA(_) => 0xe0,
            LdhPlusCFromA => 0xe2,
            LdAToInd(_) => 0xea,
            LdhToA(_) => 0xf0,
            LdhPlusCToA => 0xf2,
            LdAFromInd(_) => 0xfa,
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
            LdR8FromR8(GpRegister::IndHL, GpRegister::IndHL) => return Nop.into(),
            LdR8FromR8(to, from) => out[0] += (to as u8 * 0x08) + from as u8,
            LdR16Imm(r16, imm) => {
                out[0] += r16 as u8 * 0x10;
                out.extend(imm.to_le_bytes());
            }
            LdAFromR16(r16)
            | LdAToR16(r16) => out[0] += r16 as u8 * 0x10,
            LdR8Imm(r8, imm) => {
                out[0] += r8 as u8 * 0x08;
                out.push(imm);
            }
            LdhFromA(imm)
            | LdhToA(imm) => out.push(imm),
            LdAFromInd(imm)
            | LdAToInd(imm) => out.extend(imm.to_le_bytes()),
            IncR8(r8)
            | DecR8(r8)
            | Cp(r8) => out[0] += r8 as u8 * 0x08,
            IncR16(r16)
            | DecR16(r16) => out[0] += r16 as u8 * 0x10,
            ref v @ Jp(condition, _)
            | ref v @ Jr(condition, _) => {
                out[0] += match condition {
                    Condition::Always => 0,
                    Condition::Flag(flag) => flag as u8 * 0x08,
                };

                match v {
                    Jp(_, imm) => out.extend(imm.to_le_bytes()),
                    Jr(_, imm) => out.push(*imm as u8),
                    _ => unreachable!("Filtered down to just Jp|Jr in the outer match")
                }
            },
            Push(r16)
            | Pop(r16) => out[0] += r16 as u8 * 0x10,
            Prefixed(instruction) => out.push(instruction.into()),
            Nop
            | LdhPlusCFromA
            | LdhPlusCToA => {},
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

impl From<PrefixInstruction> for Instruction {
    fn from(value: PrefixInstruction) -> Self {
        Self::Prefixed(value)
    }
}