pub mod instructions;

pub use instructions::Condition;

use crate::codegen::AssemblerError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpRegister {
    B, C,
    D, E,
    H, L,
    IndHL,
    A,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuFlag {
    NZ, Z,
    NC, C,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitError {
    StackPointer
}

impl From<SplitError> for AssemblerError {
    fn from(value: SplitError) -> Self {
        Self::RegSplitError(value)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterPair {
    BC, DE,
    HL, SP,
}

impl RegisterPair {
    pub fn try_split(&self) -> Result<(GpRegister, GpRegister), SplitError> {
        use RegisterPair::*;
        use GpRegister::*;
        match self {
            BC => Ok((B, C)),
            DE => Ok((D, E)),
            HL => Ok((H, L)),
            SP => Err(SplitError::StackPointer),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackPair {
    BC, DE,
    HL, AF,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndirectPair {
    BC, DE,
    HLInc, HLDec,
}