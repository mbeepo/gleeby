use std::{cell::{RefCell, RefMut}, hash::Hash, rc::Rc};

use crate::{codegen::allocator::RegKind, cpu::{CpuFlag, GpRegister, IndirectPair, RegisterPair, SplitError, StackPair}, memory::Addr};

use super::{allocator::{AllocErrorTrait, Allocator, ConstAllocator, GpRegisters, RcGpRegister, RcRegisterPair}, assembler::ErrorTrait, meta_instr::{MetaInstructionTrait, VarOrConst}, Assembler};

pub(crate) type IdInner = usize;

/// Identifier for variables
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    #[default]
    Unset,
    Set(IdInner),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Unallocated { len: u16, id: Id },
    Reg(RegVariable),
    Memory(MemoryVariable),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoRcVariable {
    Unallocated { len: u16, id: Id },
    Reg(NoRcRegVariable),
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
pub enum NoRcRegVariable {
    UnallocatedR8(Id),
    UnallocatedR16(Id),
    R8 { reg: GpRegister, id: Id },
    R16 { reg_pair: RegisterPair, id: Id },
    MemR8 { addr: Addr, reg: GpRegister, id: Id },
    MemR16 { addr: Addr, reg_pair: RegisterPair, id: Id },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegVariable {
    Rc(RcRegVariable),
    NoRc(NoRcRegVariable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryVariable { pub addr: Addr, pub len: u16, pub id: Id }

impl From<RcRegVariable> for Variable {
    fn from(value: RcRegVariable) -> Self {
        Self::Reg(RegVariable::Rc(value))
    }
}

impl From<GpRegister> for NoRcRegVariable {
    fn from(value: GpRegister) -> Self {
        Self::R8 { reg: value, id: Id::Unset }
    }
}

impl From<RegisterPair> for NoRcRegVariable {
    fn from(value: RegisterPair) -> Self {
        Self::R16 { reg_pair: value, id: Id::Unset }
    }
}

impl Variable {
    /// **Prevents this register from being automatically deallocated**
    /// 
    /// Releases this register's reference count
    /// 
    /// 
    /// This is like telling gleeby's register allocator to just trust you
    pub fn no_rc(self) -> NoRcVariable {
        match self {
            Variable::Unallocated { len, id } => Self::Unallocated { len, id },
            Variable::Reg(value) => match value {
                RegVariable::Rc(var) => {
                    let h = var.allocator.borrow()[var.inner];
                    var.allocator.borrow_mut()[var.inner] = (h.0, None);
                    var.inner
                },
                RegVariable::NoRc(var) => Self::Reg(var),
            }
            Variable::Memory(var) => Self::Memory(var),
        }
    }
}

impl From<NoRcVariable> for Variable {
    fn from(value: NoRcVariable) -> Self {
        match value {
            NoRcVariable::Unallocated { len, id } => Self::Unallocated { len, id },
            NoRcVariable::Reg(var) => Self::Reg(var),
            NoRcVariable::Memory(var) => Self::Memory(var),
        }
    }
}

impl From<NoRcRegVariable> for NoRcVariable {
    fn from(value: NoRcRegVariable) -> Self {
        Self::Reg(value)
    }
}

impl From<MemoryVariable> for NoRcVariable {
    fn from(value: MemoryVariable) -> Self {
        Self::Memory(value)
    }
}

impl From<NoRcRegVariable> for Variable {
    fn from(value: NoRcRegVariable) -> Self {
        Self::Reg(value)
    }
}

impl From<MemoryVariable> for Variable {
    fn from(value: MemoryVariable) -> Self {
        Self::Memory(value)
    }
}

impl From<RcRegVariable> for RegVariable {
    fn from(value: RcRegVariable) -> Self {
        Self::Rc(value)
    }
}

impl From<NoRcRegVariable> for RegVariable {
    fn from(value: NoRcRegVariable) -> Self {
        Self::NoRc(value)
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
    fn allocator(&self) -> Rc<RefCell<Self::Alloc>>;
    fn allocator_mut(&mut self) -> RefMut<Self::Alloc>;

    fn load_var(&mut self, var: &Variable) -> Result<RegVariable, Error> {
        let allocator = self.allocator();
        let out: RegVariable = match var {
            Variable::Memory(var) => {
                match RegKind::<AllocError>::try_from_len(var.len)? {
                    RegKind::GpRegister => {
                        if let Ok(reg) = allocator.borrow_mut().alloc_reg().map(|i| i.no_rc()) {
                            if reg != GpRegister::A {
                                // will have to change which register the variable refers to that previously referred to `a`
                                self.ld_r8_from_r8(reg, GpRegister::A);
                            }

                            self.ld_a_from_ind(var.addr);
                            NoRcRegVariable::MemR8 { addr: var.addr, reg, id: var.id }.into()
                        } else {
                            todo!("Swap variable to memory")
                        }
                    }
                    RegKind::RegisterPair => {
                        let allocated = allocator.borrow_mut().alloc_reg_pair().map(|i| i.no_rc());
                        if let Ok(reg_pair) = allocated {
                            let (reg1, reg2): (GpRegister, GpRegister) = reg_pair.try_split()?;
                            let tmp = allocator.borrow_mut().alloc_reg().map(|i| i.no_rc());
                            let reg_a = allocator.borrow().get_reg(GpRegister::A).no_rc();

                            // swap the current value of `a` into a temporary register
                            let (swapped, tmp_reg) = if let Ok(tmp) = tmp {
                                if tmp != GpRegister::A {
                                    self.ld_r8_from_r8(tmp, reg_a);
                                }

                                (true, Some(tmp))
                            } else { (false, None) };

                            // 40t cycles
                            // 8 bytes
                            self.ld_a_from_ind(var.addr);
                            self.ld_r8_from_r8(reg2, reg_a);
                            self.ld_a_from_ind(var.addr + 1);
                            self.ld_r8_from_r8(reg1, reg_a);

                            // swap the old value back into `a`
                            if swapped {
                                let tmp_reg = tmp_reg.unwrap();
                                self.ld_r8_from_r8(reg_a, tmp_reg);
                                allocator.borrow_mut().release_reg(tmp_reg.into());
                            }

                            NoRcRegVariable::MemR16 { addr: var.addr, reg_pair, id: var.id }.into()
                        } else {
                            dbg!(allocator.borrow());
                            todo!("Swap variable to memory")
                        }
                    }
                    _ => unreachable!("The typechecker will never win")
                }
            }
            Variable::RcReg(var) => var.clone().into(),
            Variable::Reg(var) => var,
            Variable::Unallocated { len, id } => {
                if *len == 1 {
                    RcRegVariable::UnallocatedR8(*id)
                } else if *len == 2 {
                    RcRegVariable::UnallocatedR16(*id)
                } else {
                    Err(AllocError::oversized_load())?
                }
            }
        };

        Ok(out)
    }

    fn store_var(&mut self, var: Variable) -> Result<MemoryVariable, Error> {
        let allocator = self.allocator();
        let out = match var {
            Variable::Memory(var) => var,
            Variable::RcReg(reg) => {
                match reg {
                    RcRegVariable::R8 { reg, id } => {
                        let addr = allocator.borrow_mut().alloc_var(1)?;
                        let tmp = allocator.borrow_mut().alloc_reg();

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
                    RcRegVariable::R16 { reg_pair: _, id: _ } => {
                        todo!()
                    }
                    RcRegVariable::MemR8 { addr, reg: _, id } => MemoryVariable { addr, len: 1, id },
                    RcRegVariable::MemR16 { addr, reg_pair: _, id } => MemoryVariable { addr, len: 2, id },
                    RcRegVariable::UnallocatedR8(_)
                    | RcRegVariable::UnallocatedR16(_) => {
                        unimplemented!()
                    }
                }
            }
            Variable::Unallocated { .. } => unimplemented!()
        };

        Ok(out)
    }

    fn set_var(&mut self, var: &mut Variable, value: &mut VarOrConst) -> Result<&mut Self, Error> {
        let allocator = self.allocator();
        let dest = match var {
            Variable::RcReg(var) => var.clone(),
            Variable::Memory(var) => match RegKind::<AllocError>::try_from_len(var.len) {
                Ok(RegKind::GpRegister) => RcRegVariable::MemR8 { reg: allocator.borrow_mut().alloc_reg()?, addr: var.addr, id: var.id },
                Ok(RegKind::RegisterPair) => RcRegVariable::MemR16 { reg_pair: allocator.borrow_mut().alloc_reg_pair()?, addr: var.addr, id: var.id },
                _ => panic!("Variable too long to set like this"),
            },
            Variable::Unallocated { len, id } => match RegKind::<AllocError>::try_from_len(*len) {
                Ok(RegKind::GpRegister) => {
                    let out = RcRegVariable::R8 { reg: allocator.borrow_mut().alloc_reg()?, id: *id };
                    *var = out.clone().into();
                    out
                },
                Ok(RegKind::RegisterPair) => {
                    let out = RcRegVariable::R16 { reg_pair: allocator.borrow_mut().alloc_reg_pair()?, id: *id };
                    *var = out.clone().into();
                    out
                },
                _ => panic!("Variable too long to set like this"),
            }
        };

        match value {
            VarOrConst::Var(src_var) => {
                let src = self.load_var(src_var)?;
                match (dest, src) {
                    (RcRegVariable::R8 { reg: dest, ..} | RcRegVariable::MemR8 { reg: dest, .. },
                        RcRegVariable::R8 { reg: src, .. } | RcRegVariable::MemR8 { reg: src, .. }) => { self.ld_r8_from_r8(&dest, &src); },
                    (RcRegVariable::R16 { reg_pair: dest, ..} | RcRegVariable::MemR16 { reg_pair: dest, .. },
                        RcRegVariable::R16 { reg_pair: src, .. } | RcRegVariable::MemR16 { reg_pair: src, .. }) => {
                            let (dest1, dest2) = dest.try_split()?;
                            let (src1, src2) = src.try_split()?;

                            self.ld_r8_from_r8(&dest1, &src1);
                            self.ld_r8_from_r8(&dest2, &src2);
                        },
                    (RcRegVariable::UnallocatedR8(id), _) => {
                        let addr = allocator.borrow_mut().alloc_var(1)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });

                        self.set_var(&mut reg, value)?;
                        *var = reg;
                    }
                    (RcRegVariable::UnallocatedR16(id), _) => {
                        let addr = allocator.borrow_mut().alloc_var(2)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 2, id });

                        self.set_var(&mut reg, value)?;
                        *var = reg
                    }
                    (_, RcRegVariable::UnallocatedR8(_)| RcRegVariable::UnallocatedR16(_)) => { self.meta(Meta::set_var(var.clone(), value.clone())); },
                    (RcRegVariable::R8 { .. } | RcRegVariable::MemR8 { .. },
                        RcRegVariable::R16 { .. } | RcRegVariable::MemR16 { .. })
                    | (RcRegVariable::R16 { .. } | RcRegVariable::MemR16 { .. },
                        RcRegVariable::R8 { .. } | RcRegVariable::MemR8 { .. }) => todo!()
                };
            }
            VarOrConst::Const(src_const) => {
                match (dest, src_const) {
                    (RcRegVariable::R8 { reg: dest, ..} | RcRegVariable::MemR8 { reg: dest, .. },
                        Constant::Inline8(src)) => { self.ld_r8_imm(&dest, *src); },
                    (RcRegVariable::R16 { reg_pair: dest, .. } | RcRegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Inline16(src)) => { self.ld_r16_imm(&dest, *src); },
                    (RcRegVariable::R16 { reg_pair: dest, .. } | RcRegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Addr(constant)) => { self.ld_r16_imm(&dest, constant.addr); },
                    (RcRegVariable::UnallocatedR8(id), _) => {
                        let addr = allocator.borrow_mut().alloc_var(1)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });

                        self.set_var(&mut reg, value)?;
                        *var = reg;
                    }
                    (RcRegVariable::UnallocatedR16(id), _) => {
                        let addr = allocator.borrow_mut().alloc_var(2)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 2, id });

                        self.set_var(&mut reg, value)?;
                        *var = reg;
                    }
                    _ => panic!("Heck"),
                }
            }
        }

        Ok(self)
    }

    fn dec_var(&mut self, var: &Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RcRegVariable::R8 { reg, .. }
            | RcRegVariable::MemR8 { reg, .. } => self.dec_r8(&reg),
            RcRegVariable::R16 { reg_pair, .. }
            | RcRegVariable::MemR16 { reg_pair, .. }  => self.dec_r16(&reg_pair),
            RcRegVariable::UnallocatedR8(_)
            | RcRegVariable::UnallocatedR16(_) => self.meta(Meta::dec_var(var.clone())),
        };

        Ok(self)
    }

    fn inc_var(&mut self, var: &Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RcRegVariable::R8 { reg, .. }
            | RcRegVariable::MemR8 { reg, .. } => self.inc_r8(&reg),
            RcRegVariable::R16 { reg_pair, .. }
            | RcRegVariable::MemR16 { reg_pair, .. }  => self.inc_r16(&reg_pair),
            RcRegVariable::UnallocatedR8(_)
            | RcRegVariable::UnallocatedR16(_) => self.meta(Meta::inc_var(var.clone())),
        };

        Ok(self)
    }

    fn ld_a_from_var_ind(&mut self, var: &Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RcRegVariable::R8 { reg: RcGpRegister { inner: GpRegister::C, .. }, .. }
            | RcRegVariable::MemR8 { reg: RcGpRegister { inner: GpRegister::C, .. }, .. } => { self.ldh_from_a_with_c(); },
            RcRegVariable::R8 { .. }
            | RcRegVariable::MemR8 { .. } => Err(Error::invalid_arg())?,
            RcRegVariable::R16 { reg_pair, .. }
            | RcRegVariable::MemR16 { reg_pair, .. } => {
                if let Ok(ref pair) = reg_pair.inner.try_into() {
                    self.ld_a_from_r16(pair);
                } else {
                    Err(Error::invalid_arg())?;
                }
            },
            RcRegVariable::UnallocatedR8(_)
            | RcRegVariable::UnallocatedR16(_) => { 
                let reg_a: RcRegVariable = self.allocator().borrow().get_reg(GpRegister::A).into();
                self.meta(Meta::var_from_ind(reg_a.into(), var.clone()));
            },
        };

        Ok(self)
    }

    fn jr_z_var(&mut self, var: &Variable, imm: i8) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RcRegVariable::R8 { reg, .. }
            | RcRegVariable::MemR8 { reg, .. } => {
                let tmp = self.allocator_mut().alloc_reg()?;
                if reg == GpRegister::A {
                    self.cp(&tmp);
                } else if tmp == GpRegister::A {
                    self.cp(&reg);
                }
                self.jr(CpuFlag::Z.into(), imm);
            }
            _ => todo!()
        }

        Ok(self)
    }
}