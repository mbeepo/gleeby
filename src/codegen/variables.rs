use std::hash::Hash;

use crate::{codegen::allocator::RegKind, cpu::{GpRegister, IndirectPair, RegisterPair, SplitError, StackPair}, memory::Addr};

use super::{allocator::{AllocErrorTrait, Allocator}, assembler::ErrorTrait, meta_instr::{MetaInstructionTrait, VarOrConst}, Assembler};

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
pub struct StoredConstant {
    pub id: Id,
    pub addr: Addr,
    pub len: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constant {
    Inline8(u8),
    Inline16(u16),
    Addr(StoredConstant)
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

impl From<RegVariable> for Variable {
    fn from(value: RegVariable) -> Self {
        Self::Reg(value)
    }
}

impl From<MemoryVariable> for Variable {
    fn from(value: MemoryVariable) -> Self {
        Self::Memory(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegSelector {
    R8(GpRegister),
    R16(RegisterPair),
}

impl From<GpRegister> for RegSelector {
    fn from(value: GpRegister) -> Self {
        Self::R8(value)
    }
}

impl From<RegisterPair> for RegSelector {
    fn from(value: RegisterPair) -> Self {
        Self::R16(value)
    }
}

pub trait Variabler<Meta, Error, AllocError>: Assembler<Meta>
        where Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError> + ErrorTrait,
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait,
            Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    type Alloc: Allocator<AllocError>;

    fn new_var(&mut self, len: u16) -> Variable;
    fn allocator(&mut self) -> &mut Self::Alloc;

    fn load_var(&mut self, var: &mut Variable) -> Result<RegVariable, Error> {
        let out = match var {
            Variable::Memory(var) => {
                match RegKind::<AllocError>::try_from_len(var.len)? {
                    RegKind::GpRegister => {
                        if !self.allocator().reg_is_used(GpRegister::A.into()) {
                            self.allocator().claim_reg(GpRegister::A.into(), Id::Set(0));
                        }
                        if let Ok(reg) = self.allocator().alloc_reg() {
                            if reg != GpRegister::A {
                                // will have to change which register the variable refers to that previously referred to `a`
                                self.ld_r8_from_r8(reg, GpRegister::A);
                            }
                            
                            self.ld_a_from_ind(var.addr);
                            RegVariable::MemR8 { addr: var.addr, reg, id: var.id }
                        } else {
                            todo!("Swap variable to memory")
                        }
                    }
                    RegKind::RegisterPair => {
                        if let Ok(reg_pair) = self.allocator().alloc_reg_pair() {
                            let (reg1, reg2): (GpRegister, GpRegister) = reg_pair.try_split()?;
                            let tmp = self.allocator().alloc_reg();

                            let stacked = if let Ok(tmp) = tmp {
                                if tmp != GpRegister::A {
                                    // will have to change which register the variable refers to that previously referred to `a`
                                    self.ld_r8_from_r8(tmp, GpRegister::A);
                                }

                                false
                            } else {
                                self.push(StackPair::AF);
                                true
                            };
                            
                            // 40t cycles
                            // 8 bytes
                            self.ld_a_from_ind(var.addr);
                            self.ld_r8_from_r8(reg2, GpRegister::A);
                            self.ld_a_from_ind(var.addr + 1);
                            self.ld_r8_from_r8(reg1, GpRegister::A);

                            if stacked {
                                self.pop(StackPair::AF);
                            } else {
                                self.allocator().release_reg(GpRegister::A.into());
                            }

                            RegVariable::MemR16 { addr: var.addr, reg_pair, id: var.id }
                        } else {
                            todo!("Swap variable to memory")
                        }
                    }
                    _ => unreachable!("The typechecker will never win")
                }
            }
            Variable::Reg(var) => *var,
            Variable::Unallocated { len, id } => {
                if *len == 1 {
                    RegVariable::UnallocatedR8(*id)
                } else if *len == 2 {
                    RegVariable::UnallocatedR16(*id)
                } else {
                    Err(AllocError::oversized_load())?
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
                        let tmp = self.allocator().alloc_reg();

                        if let Ok(tmp) = tmp {
                            if tmp == GpRegister::A {
                                todo!()
                            } else {
                                todo!()
                            }
                        } else {
                            // fall back to `self.ld_a_to_r16`
                            todo!()
                        }
                    }
                    RegVariable::R16 { reg_pair: _, id: _ } => {
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

    fn set_var(&mut self, var: &mut Variable, value: &mut VarOrConst) -> Result<&mut Self, Error> {
        let dest = self.load_var(var)?;
        match value {
            VarOrConst::Var(src_var) => {
                let src = self.load_var(src_var)?;
                match (dest, src) {
                    (RegVariable::R8 { reg: dest, ..} | RegVariable::MemR8 { reg: dest, .. },
                        RegVariable::R8 { reg: src, .. } | RegVariable::MemR8 { reg: src, .. }) => { self.ld_r8_from_r8(dest, src); },
                    (RegVariable::R16 { reg_pair: dest, ..} | RegVariable::MemR16 { reg_pair: dest, .. },
                        RegVariable::R16 { reg_pair: src, .. } | RegVariable::MemR16 { reg_pair: src, .. }) => {
                            let (dest1, dest2) = dest.try_split()?;
                            let (src1, src2) = src.try_split()?;

                            self.ld_r8_from_r8(dest1, src1);
                            self.ld_r8_from_r8(dest2, src2);
                        },
                    (RegVariable::UnallocatedR8(id), _) => {
                        let addr = self.allocator().alloc_var(1)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });
                        let reg = self.load_var(&mut reg)?;

                        self.set_var(&mut reg.into(), value)?;
                        *var = reg.into();
                    }
                    (RegVariable::UnallocatedR16(id), _) => {
                        let addr = self.allocator().alloc_var(2)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 2, id });
                        let reg = self.load_var(&mut reg)?;

                        self.set_var(&mut reg.into(), value)?;
                        *var = reg.into()
                    }
                    (_, RegVariable::UnallocatedR8(_)| RegVariable::UnallocatedR16(_)) => { self.meta(Meta::set_var(*var, *value)); },
                    (RegVariable::R8 { .. } | RegVariable::MemR8 { .. },
                        RegVariable::R16 { .. } | RegVariable::MemR16 { .. })
                    | (RegVariable::R16 { .. } | RegVariable::MemR16 { .. },
                        RegVariable::R8 { .. } | RegVariable::MemR8 { .. }) => todo!()
                };
            }
            VarOrConst::Const(src_const) => {
                match (dest, src_const) {
                    (RegVariable::R8 { reg: dest, ..} | RegVariable::MemR8 { reg: dest, .. },
                        Constant::Inline8(src)) => { self.ld_r8_imm(dest, *src); },
                    (RegVariable::R16 { reg_pair: dest, .. } | RegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Inline16(src)) => { self.ld_r16_imm(dest, *src); },
                    (RegVariable::R16 { reg_pair: dest, .. } | RegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Addr(constant)) => { self.ld_r16_imm(dest, constant.addr); },
                    (RegVariable::UnallocatedR8(id), _) => {
                        let addr = self.allocator().alloc_var(1)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });
                        let reg = self.load_var(&mut reg)?;

                        self.set_var(&mut reg.into(), value)?;
                        *var = reg.into();
                    }
                    (RegVariable::UnallocatedR16(id), _) => {
                        let addr = self.allocator().alloc_var(2)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 2, id });
                        let reg = self.load_var(&mut reg)?;

                        self.set_var(&mut reg.into(), value)?;
                        *var = reg.into();
                    }
                    _ => panic!("Heck"),
                }
            }
        }

        Ok(self)
    }

    fn dec_var(&mut self, var: &mut Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::R8 { reg, .. }
            | RegVariable::MemR8 { reg, .. } => self.dec_r8(reg),
            RegVariable::R16 { reg_pair, .. }
            | RegVariable::MemR16 { reg_pair, .. }  => self.dec_r16(reg_pair),
            RegVariable::UnallocatedR8(_)
            | RegVariable::UnallocatedR16(_) => self.meta(Meta::dec_var(*var)),
        };

        Ok(self)
    }

    fn inc_var(&mut self, var: &mut Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::R8 { reg, .. }
            | RegVariable::MemR8 { reg, .. } => self.inc_r8(reg),
            RegVariable::R16 { reg_pair, .. }
            | RegVariable::MemR16 { reg_pair, .. }  => self.inc_r16(reg_pair),
            RegVariable::UnallocatedR8(_)
            | RegVariable::UnallocatedR16(_) => self.meta(Meta::inc_var(*var)),
        };

        Ok(self)
    }

    fn ld_a_from_var_ind(&mut self, var: &mut Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::R8 { reg: GpRegister::C, .. }
            | RegVariable::MemR8 { reg: GpRegister::C, .. } => { self.ldh_from_a_with_c(); },
            RegVariable::R8 { .. }
            | RegVariable::MemR8 { .. } => Err(Error::invalid_arg())?,
            RegVariable::R16 { reg_pair, .. }
            | RegVariable::MemR16 { reg_pair, .. } => {
                if let Ok(pair) = reg_pair.try_into() {
                    self.ld_a_from_r16(pair);
                } else {
                    Err(Error::invalid_arg())?;
                }
            },
            RegVariable::UnallocatedR8(_)
            | RegVariable::UnallocatedR16(_) => { self.meta(Meta::var_from_ind(RegVariable::R8 { reg: GpRegister::A, id: Id::Unset }.into(), *var)); },
        };

        Ok(self)
    }
}