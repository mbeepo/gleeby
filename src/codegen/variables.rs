use crate::{cpu::{Register, RegisterPair}, memory::Addr};

pub(crate) type IdInner = usize;

/// Identifier for variables
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Id {
    #[default]
    Unset,
    Set(IdInner),
}

/// [Id] but for [Block]s
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Ctx {
    #[default]
    Unset,
    Set(IdInner),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variable {
    Dynamic { id: Id, ctx: Ctx },
    StaticR8(Register),
    StaticR16(RegisterPair),
    StaticAddr(Addr),
}

impl Default for Variable {
    fn default() -> Self {
        Self::Dynamic { id: Default::default(), ctx: Default::default() }
    }
}