pub mod instructions;

pub use instructions::Condition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B, C,
    D, E,
    H, L,
    IndHL,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterPair {
    BC, DE,
    HL, SP,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackPair {
    BC, DE,
    HL, AF,
}