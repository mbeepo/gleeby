use std::hash::Hash;

use crate::{cpu::{GpRegister, RegisterPair, SplitError, StackPair}, memory::Addr};

use super::{allocator::Allocator, assembler::AsBuf, Assembler};

pub(crate) type IdInner = usize;

/// Identifier for variables
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    #[default]
    Unset,
    Set(IdInner),
}

/// [Id] but for [Block]s
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Ctx {
    #[default]
    Unset,
    Set(IdInner),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variable {
    Unallocated { id: Id, ctx: Ctx },
    Allocated(ConfirmedVariable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmedVariable {
    Reg(RegVariable),
    Memory(MemoryVariable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegVariable {
    R8 { reg: GpRegister, id: Id, ctx: Ctx },
    R16 { reg_pair: RegisterPair, id: Id, ctx: Ctx },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryVariable { pub addr: Addr, pub len: u16, pub id: Id, pub ctx: Ctx }

impl Variable {
    pub fn new() -> Self {
        Self::Unallocated { id: Default::default(), ctx: Default::default() }
    }
}

pub trait Variabler<Error, AllocError>: Assembler
        where Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError>,
            AllocError: Clone + std::fmt::Debug + Into<Error> {
    type Alloc: Allocator<AllocError>;

    fn new_var<T>(&mut self, var: T) -> Variable
        where T: AsBuf;
    fn allocator(&mut self) -> &mut Self::Alloc;


    fn load_var(&mut self, var: MemoryVariable) -> Result<RegVariable, Error> {
        let reg = self.allocator().alloc_reg(ConfirmedVariable::Memory(var))?;
        match reg {
            RegVariable::R8 { reg, .. } => {
                self.push(StackPair::HL);
                self.ld_r16_imm(RegisterPair::HL, var.addr);
                self.ld_r8_from_r8(reg, GpRegister::IndHL);
                self.pop(StackPair::HL);
            },
            RegVariable::R16 { reg_pair, .. } => {
                let (reg1, reg2): (GpRegister, GpRegister) = reg_pair.try_split()?;

                if reg_pair != RegisterPair::HL {
                    self.ld_a_from_ind16(var.addr);
                    self.ld_r8_from_r8(reg2, GpRegister::A);
                    self.ld_a_from_ind16(var.addr + 1);
                    self.ld_r8_from_r8(reg1, GpRegister::A);
                } else {
                    self.push(StackPair::HL);
                    self.ld_r16_imm(RegisterPair::HL, var.addr);
                    self.ld_r8_from_r8(reg2, GpRegister::IndHL);
                    self.inc_r16(RegisterPair::HL);
                    self.ld_r8_from_r8(reg1, GpRegister::IndHL);
                    self.pop(StackPair::HL);
                }
            }
        }

        todo!()
    }

    fn store_var(&mut self, var: RegVariable) {
        todo!()
    }

    fn decrement_var(&mut self, var: ConfirmedVariable) -> Result<&mut Self, Error> {
        let var = match var {
            ConfirmedVariable::Reg(var) => var,
            ConfirmedVariable::Memory(var) => self.load_var(var)?,
        };

        match var {
            RegVariable::R8 { reg, .. } => self.dec_r8(reg),
            RegVariable::R16 { reg_pair, .. }  => self.dec_r16(reg_pair),
        };

        Ok(self)
    }
}