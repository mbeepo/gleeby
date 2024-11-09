use std::hash::Hash;

use crate::{codegen::allocator::RegKind, cpu::{GpRegister, RegisterPair, SplitError, StackPair}, memory::Addr};

use super::{allocator::{AllocErrorTrait, Allocator}, meta_instr::MetaInstructionTrait, Assembler};

pub(crate) type IdInner = usize;

/// Identifier for variables
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    #[default]
    Unset,
    Set(IdInner),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variable {
    Unallocated { len: u16, id: Id },
    Reg(RegVariable),
    Memory(MemoryVariable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegVariable {
    UnallocatedR8(Id),
    UnallocatedR16(Id),
    R8 { reg: GpRegister, id: Id },
    R16 { reg_pair: RegisterPair, id: Id },
    MemR8 { addr: Addr, reg: GpRegister, id: Id },
    MemR16 { addr: Addr, reg_pair: RegisterPair, id: Id },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryVariable { pub addr: Addr, pub len: u16, pub id: Id }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegSelector {
    R8(GpRegister),
    R16(RegisterPair),
}

pub trait Variabler<Meta, Error, AllocError>: Assembler<Meta>
        where Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError>,
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait,
            Meta: Clone + std::fmt::Debug + MetaInstructionTrait + MetaInstructionTrait {
    type Alloc: Allocator<AllocError>;

    fn new_var(&mut self, len: u16) -> Variable;
    fn allocator(&mut self) -> &mut Self::Alloc;


    fn load_var(&mut self, var: Variable) -> Result<RegVariable, Error> {
        let out = match var {
            Variable::Memory(var) => {
                

                match reg {
                    RegSelector::R8(reg) => {
                        self.push(StackPair::HL);
                        self.ld_r16_imm(RegisterPair::HL, var.addr);
                        self.ld_r8_from_r8(reg, GpRegister::IndHL);
                        self.pop(StackPair::HL);

                        RegVariable::MemR8 { addr: var.addr, reg, id: var.id }
                    },
                    RegSelector::R16(reg_pair) => {
                        let (reg1, reg2): (GpRegister, GpRegister) = reg_pair.try_split()?;

                        if reg_pair != RegisterPair::HL {
                            self.ld_a_from_ind(var.addr);
                            self.ld_r8_from_r8(reg2, GpRegister::A);
                            self.ld_a_from_ind(var.addr + 1);
                            self.ld_r8_from_r8(reg1, GpRegister::A);
                        } else {
                            self.push(StackPair::HL);
                            self.ld_r16_imm(RegisterPair::HL, var.addr);
                            self.ld_r8_from_r8(reg2, GpRegister::IndHL);
                            self.inc_r16(RegisterPair::HL);
                            self.ld_r8_from_r8(reg1, GpRegister::IndHL);
                            self.pop(StackPair::HL);
                        }

                        RegVariable::MemR16 { addr: var.addr, reg_pair, id: var.id }
                    },
                }
            }
            Variable::Reg(var) => var,
            Variable::Unallocated { len, id } => {
                if len == 1 {
                    RegVariable::UnallocatedR8(id)
                } else if len == 2 {
                    RegVariable::UnallocatedR16(id)
                } else {
                    Err(AllocError::oversized_reg())?
                }
            }
        };

        Ok(out)
    }

    fn store_var(&mut self, var: Variable) -> Result<MemoryVariable, Error> {
        let out = match var {
            Variable::Memory(var) => var,
            Variable::Reg(reg) => {
                match reg {
                    RegVariable::R8 { reg, id } => {
                        let addr = self.allocator().alloc_var(1)?;
                        let tmp = self.allocator().alloc_reg(RegKind::RegisterPair);

                        todo!()
                    }
                    RegVariable::R16 { reg_pair, id } => {
                        todo!()
                    }
                    RegVariable::MemR8 { addr, reg: _, id } => MemoryVariable { addr, len: 1, id },
                    RegVariable::MemR16 { addr, reg_pair: _, id } => MemoryVariable { addr, len: 2, id },
                    RegVariable::UnallocatedR8(_)
                    | RegVariable::UnallocatedR16(_) => {
                        unimplemented!()
                    }
                }
            }
            Variable::Unallocated { .. } => unimplemented!()
        };

        Ok(out)
    }

    fn dec_var(&mut self, var: Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::R8 { reg, .. }
            | RegVariable::MemR8 { reg, .. } => self.dec_r8(reg),
            RegVariable::R16 { reg_pair, .. }
            | RegVariable::MemR16 { reg_pair, .. }  => self.dec_r16(reg_pair),
            RegVariable::UnallocatedR8(_)
            | RegVariable::UnallocatedR16(_) => self.meta(Meta::var_dec(var)),
        };

        Ok(self)
    }
}