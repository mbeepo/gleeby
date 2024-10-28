pub mod instructions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B, C,
    D, E,
    H, L,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterPair {
    BC,
    DE,
    HL,
}
