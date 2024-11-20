use std::{cell::RefCell, fmt::Display, hash::Hash, rc::Rc};

use crate::{codegen::allocator::RegKind, cpu::{CpuFlag, GpRegister, RegisterPair, SplitError, StackPair}, memory::Addr};

use super::{allocator::{AllocErrorTrait, Allocator, RcRegVariable}, assembler::ErrorTrait, meta_instr::{MetaInstructionTrait, VarOrConst}, Assembler};

pub(crate) type IdInner = usize;

/// Identifier for variables
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    #[default]
    Unset,
    Set(IdInner),
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Unset => "UnsetId".to_owned(),
            Self::Set(id) => format!("{}", id),
        };

        write!(f, "{}", out)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Unallocated { len: u16, id: Id },
    Reg(RegVariable),
    Memory(MemoryVariable),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawVariable {
    Unallocated { len: u16, id: Id },
    Reg(RawRegVariable),
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
pub enum RawRegVariable {
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
    Raw(RawRegVariable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryVariable { pub addr: Addr, pub len: u16, pub id: Id }

impl Variable {
    /// **Prevents this register from being automatically deallocated**
    /// 
    /// Releases this register's reference count
    /// 
    /// 
    /// This is like telling gleeby's register allocator to just trust you
    pub fn into_raw(self) -> RawVariable {
        match self {
            Variable::Unallocated { len, id } => RawVariable::Unallocated { len, id },
            Variable::Reg(value) => match value {
                RegVariable::Rc(var) => RawVariable::Reg(var.into_raw()),
                RegVariable::Raw(var) => RawVariable::Reg(var.into()),
            }
            Variable::Memory(var) => RawVariable::Memory(var),
        }
    }
}

impl From<RawVariable> for Variable {
    fn from(value: RawVariable) -> Self {
        match value {
            RawVariable::Unallocated { len, id } => Self::Unallocated { len, id },
            RawVariable::Reg(var) => Self::Reg(var.into()),
            RawVariable::Memory(var) => Self::Memory(var),
        }
    }
}

impl From<Variable> for RawVariable {
    fn from(value: Variable) -> Self {
        match value {
            Variable::Reg(RegVariable::Rc( RcRegVariable { inner, .. }))
            | Variable::Reg(RegVariable::Raw(inner)) => Self::Reg(inner),
            Variable::Memory(var) => Self::Memory(var),
            Variable::Unallocated { len, id } => Self::Unallocated { len, id }
        }
    }
}

impl<T> From<T> for Variable where T: Into<RegVariable> {
    fn from(value: T) -> Self {
        Self::Reg(value.into())
    }
}

impl From<RawRegVariable> for RawVariable {
    fn from(value: RawRegVariable) -> Self {
        Self::Reg(value)
    }
}

impl From<MemoryVariable> for RawVariable {
    fn from(value: MemoryVariable) -> Self {
        Self::Memory(value)
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

impl From<RawRegVariable> for RegVariable {
    fn from(value: RawRegVariable) -> Self {
        Self::Raw(value)
    }
}

impl From<GpRegister> for RawRegVariable {
    fn from(value: GpRegister) -> Self {
        Self::R8 { reg: value, id: Id::Unset }
    }
}

impl From<RegisterPair> for RawRegVariable {
    fn from(value: RegisterPair) -> Self {
        Self::R16 { reg_pair: value, id: Id::Unset }
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

    fn load_var(&mut self, var: &Variable) -> Result<RegVariable, Error> {
        let allocator = self.allocator();
        let out: RegVariable = match var {
            Variable::Memory(var) => {
                match RegKind::<AllocError>::try_from_len(var.len)? {
                    RegKind::GpRegister => {
                        if let Ok(reg) = allocator.borrow_mut().alloc_reg() {
                            if reg.inner != GpRegister::A {
                                // will have to change which register the variable refers to that previously referred to `a`
                                self.ld_r8_from_r8(reg.inner, GpRegister::A);
                            }

                            self.ld_a_from_ind(var.addr);
                            RcRegVariable {
                                inner: RawRegVariable::MemR8 {
                                    addr: var.addr,
                                    reg: reg.inner,
                                    id: var.id
                                },
                                allocator: reg.allocator.clone(),
                            }.into()
                        } else {
                            todo!("Swap variable to memory")
                        }
                    }
                    RegKind::RegisterPair => {
                        let allocated = allocator.borrow_mut().alloc_reg_pair();
                        if let Ok(reg_pair) = allocated {
                            let (reg1, reg2): (GpRegister, GpRegister) = reg_pair.try_split()?;
                            let tmp = allocator.borrow_mut().alloc_reg().map(|i| i.into_raw());
                            let reg_a = GpRegister::A;

                            // swap the current value of `a` into a temporary register
                            let swapped_with = if let Ok(tmp) = tmp {
                                if tmp != GpRegister::A {
                                    self.ld_r8_from_r8(tmp, reg_a);
                                }

                                Some(tmp)
                            } else {
                                // or push to stack if no registers are free
                                self.push(&StackPair::AF);
                                None
                            };

                            // 40t cycles
                            // 8 bytes
                            self.ld_a_from_ind(var.addr);
                            self.ld_r8_from_r8(reg2, reg_a);
                            self.ld_a_from_ind(var.addr + 1);
                            self.ld_r8_from_r8(reg1, reg_a);

                            // swap the old value back into `a`
                            if let Some(tmp_reg) = swapped_with {
                                self.ld_r8_from_r8(reg_a, tmp_reg);
                                allocator.borrow_mut().release_reg(tmp_reg.into());
                            } else {
                                self.pop(&StackPair::AF);
                            }

                            RcRegVariable { 
                                inner: RawRegVariable::MemR16 {
                                    addr: var.addr,
                                    reg_pair: reg_pair.inner,
                                    id: var.id
                                },
                                allocator: reg_pair.allocator.clone()
                            }.into()
                        } else {
                            dbg!(allocator.borrow());
                            todo!("Swap variable to memory")
                        }
                    }
                    _ => unreachable!("The typechecker will never win")
                }
            }
            Variable::Reg(var @ RegVariable::Raw(_)) => var.clone(),
            Variable::Reg(RegVariable::Rc(var)) => var.inner.into(),
            Variable::Unallocated { len, id } => {
                if *len == 1 {
                    RegVariable::Raw(RawRegVariable::UnallocatedR8(*id))
                } else if *len == 2 {
                    RegVariable::Raw(RawRegVariable::UnallocatedR16(*id))
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
            Variable::Reg(RegVariable::Rc(RcRegVariable { inner: var, .. }))
            | Variable::Reg(RegVariable::Raw(var)) => {
                match var {
                    RawRegVariable::R8 { reg, id } => {
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
                    RawRegVariable::R16 { reg_pair: _, id: _ } => {
                        todo!()
                    }
                    RawRegVariable::MemR8 { addr, reg: _, id } => MemoryVariable { addr, len: 1, id },
                    RawRegVariable::MemR16 { addr, reg_pair: _, id } => MemoryVariable { addr, len: 2, id },
                    RawRegVariable::UnallocatedR8(_)
                    | RawRegVariable::UnallocatedR16(_) => {
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
        let mut new_var: Option<Variable> = None;
        let dest: RcRegVariable = match var {
            Variable::Reg(RegVariable::Rc(var)) => var.clone(),
            Variable::Reg(RegVariable::Raw(raw)) => match raw {
                RawRegVariable::R8 { reg, id }
                | RawRegVariable::MemR8 { reg, id, .. } => {
                    // get the allocator bwehehe
                    let reg = allocator.borrow_mut().claim_reg(*reg, *id);
                    RcRegVariable {
                        inner: *raw,
                        allocator: reg.allocator.clone()
                    }
                },
                RawRegVariable::R16 { reg_pair, id }
                | RawRegVariable::MemR16 { reg_pair, id, .. } => {
                    let reg_pair = allocator.borrow_mut().claim_reg_pair(*reg_pair, *id);
                    RcRegVariable {
                        inner: *raw,
                        allocator: reg_pair.allocator.clone()
                    }
                },
                RawRegVariable::UnallocatedR8(id) => {
                    let reg = allocator.borrow_mut().alloc_reg()?;
                    new_var = Some(RawRegVariable::R8 { reg: reg.inner, id: *id }.into());
                    RcRegVariable {
                        inner: RawRegVariable::R8 { reg: reg.inner, id: *id },
                        allocator: reg.allocator.clone(),
                    }
                },
                RawRegVariable::UnallocatedR16(id) => {
                    let reg = allocator.borrow_mut().alloc_reg_pair()?;
                    new_var = Some(RawRegVariable::R16 { reg_pair: reg.inner, id: *id }.into());
                    RcRegVariable {
                        inner: RawRegVariable::R16 { reg_pair: reg.inner, id: *id },
                        allocator: reg.allocator.clone(),
                    }
                },
            }
            Variable::Memory(var) => match RegKind::<AllocError>::try_from_len(var.len) {
                Ok(RegKind::GpRegister) => {
                    let reg = allocator.borrow_mut().alloc_reg()?;
                    RcRegVariable {
                        inner: RawRegVariable::MemR8 { reg: reg.inner, addr: var.addr, id: var.id },
                        allocator: reg.allocator.clone(),
                    }
                }
                Ok(RegKind::RegisterPair) => {
                    let reg = allocator.borrow_mut().alloc_reg_pair()?;
                    RcRegVariable {
                        inner: RawRegVariable::MemR16 {reg_pair: reg.inner, addr: var.addr, id: var.id },
                        allocator: reg.allocator.clone(),
                    }
                },
                _ => panic!("Variable `{}` too long to set using `set_var`", var.id),
            },
            Variable::Unallocated { len, id } => match RegKind::<AllocError>::try_from_len(*len) {
                Ok(RegKind::GpRegister) => {
                    let reg = allocator.borrow_mut().alloc_reg()?;
                    let out = RcRegVariable {
                        inner: RawRegVariable::R8 { reg: reg.inner, id: *id },
                        allocator: reg.allocator.clone(),
                    };
                    *var = out.clone().into();
                    out
                },
                Ok(RegKind::RegisterPair) => {
                    let reg = allocator.borrow_mut().alloc_reg_pair()?;
                    let out = RcRegVariable {
                        inner: RawRegVariable::R16 { reg_pair: reg.inner, id: *id },
                        allocator: reg.allocator.clone(),
                    };
                    *var = out.clone().into();
                    out
                },
                _ => panic!("Variable `{}` too long to set using `set_var`", id),
            }
        };

        if let Some(new_var) = new_var {
            *var = new_var;
        }

        match value {
            VarOrConst::Var(src_var) => {
                let src = self.load_var(src_var)?;
                match src {
                    RegVariable::Rc(RcRegVariable { inner: src, .. }) => match (dest.inner, src) {
                        (RawRegVariable::R8 { reg: dest, ..} | RawRegVariable::MemR8 { reg: dest, .. },
                            RawRegVariable::R8 { reg: src, .. } | RawRegVariable::MemR8 { reg: src, .. }) => { self.ld_r8_from_r8(dest, src); },
                        (RawRegVariable::R16 { reg_pair: dest, ..} | RawRegVariable::MemR16 { reg_pair: dest, .. },
                            RawRegVariable::R16 { reg_pair: src, .. } | RawRegVariable::MemR16 { reg_pair: src, .. }) => {
                                let (dest1, dest2) = dest.try_split()?;
                                let (src1, src2) = src.try_split()?;
    
                                self.ld_r8_from_r8(dest1, src1);
                                self.ld_r8_from_r8(dest2, src2);
                            },
                        (RawRegVariable::UnallocatedR8(id), _) => {
                            let addr = allocator.borrow_mut().alloc_var(1)?;
                            let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });
    
                            self.set_var(&mut reg, value)?;
                            *var = reg;
                        }
                        (RawRegVariable::UnallocatedR16(id), _) => {
                            let addr = allocator.borrow_mut().alloc_var(2)?;
                            let mut reg = Variable::Memory(MemoryVariable { addr, len: 2, id });
    
                            self.set_var(&mut reg, value)?;
                            *var = reg
                        }
                        (_, RawRegVariable::UnallocatedR8(_)| RawRegVariable::UnallocatedR16(_)) => { self.meta(Meta::set_var(var.clone(), value.clone())); },
                        (RawRegVariable::R8 { .. } | RawRegVariable::MemR8 { .. },
                            RawRegVariable::R16 { .. } | RawRegVariable::MemR16 { .. })
                        | (RawRegVariable::R16 { .. } | RawRegVariable::MemR16 { .. },
                            RawRegVariable::R8 { .. } | RawRegVariable::MemR8 { .. }) => todo!()
                    }
                    RegVariable::Raw(src) => todo!()
                }
            }
            VarOrConst::Const(src_const) => {
                match (dest.inner, src_const) {
                    (RawRegVariable::R8 { reg: dest, ..} | RawRegVariable::MemR8 { reg: dest, .. },
                        Constant::Inline8(src)) => { self.ld_r8_imm(dest, *src); },
                    (RawRegVariable::R16 { reg_pair: dest, .. } | RawRegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Inline16(src)) => { self.ld_r16_imm(dest, *src); },
                    (RawRegVariable::R16 { reg_pair: dest, .. } | RawRegVariable::MemR16 { reg_pair: dest, .. },
                        Constant::Addr(constant)) => { self.ld_r16_imm(dest, constant.addr); },
                    (RawRegVariable::UnallocatedR8(id), _) => {
                        let addr = allocator.borrow_mut().alloc_var(1)?;
                        let mut reg = Variable::Memory(MemoryVariable { addr, len: 1, id });

                        self.set_var(&mut reg, value)?;
                        *var = reg;
                    }
                    (RawRegVariable::UnallocatedR16(id), _) => {
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
            RegVariable::Rc(RcRegVariable { inner: var, .. })
            | RegVariable::Raw(var) => match var {
                RawRegVariable::R8 { reg, .. }
                | RawRegVariable::MemR8 { reg, .. } => self.dec_r8(reg),
                RawRegVariable::R16 { reg_pair, .. }
                | RawRegVariable::MemR16 { reg_pair, .. }  => self.dec_r16(reg_pair),
                RawRegVariable::UnallocatedR8(_)
                | RawRegVariable::UnallocatedR16(_) => self.meta(Meta::dec_var(var.into())),
            }
        };

        Ok(self)
    }

    fn inc_var(&mut self, var: &Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::Rc(RcRegVariable { inner: var, .. })
            | RegVariable::Raw(var) => match var {
                RawRegVariable::R8 { reg, .. }
                | RawRegVariable::MemR8 { reg, .. } => self.inc_r8(reg),
                RawRegVariable::R16 { reg_pair, .. }
                | RawRegVariable::MemR16 { reg_pair, .. }  => self.inc_r16(reg_pair),
                RawRegVariable::UnallocatedR8(_)
                | RawRegVariable::UnallocatedR16(_) => self.meta(Meta::inc_var(var.into())),
            }
        };

        Ok(self)
    }

    fn ld_a_from_var_ind(&mut self, var: &Variable) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::Rc(RcRegVariable { inner: var, .. })
            | RegVariable::Raw(var) => match var {
                RawRegVariable::R8 { reg: GpRegister::C, .. }
                | RawRegVariable::MemR8 { reg: GpRegister::C, .. } => { self.ldh_from_a_with_c(); },
                RawRegVariable::R8 { .. }
                | RawRegVariable::MemR8 { .. } => Err(Error::invalid_arg())?,
                RawRegVariable::R16 { reg_pair, .. }
                | RawRegVariable::MemR16 { reg_pair, .. } => {
                    if let Ok(pair) = reg_pair.try_into() {
                        self.ld_a_from_r16(pair);
                    } else {
                        Err(Error::invalid_arg())?;
                    }
                },
                RawRegVariable::UnallocatedR8(_)
                | RawRegVariable::UnallocatedR16(_) => { 
                    let reg_a = self.allocator().borrow().get_reg(GpRegister::A);
                    let raw_a: Variable =  RawRegVariable::from(reg_a.inner).into();
                    self.meta(Meta::var_from_ind(raw_a.into(), var.into()));
                },
            }
        };

        Ok(self)
    }

    fn jr_z_var(&mut self, var: &Variable, imm: i8) -> Result<&mut Self, Error> {
        let reg = self.load_var(var)?;

        match reg {
            RegVariable::Rc(RcRegVariable { inner: var, .. })
            | RegVariable::Raw(var) => match var {
                RawRegVariable::R8 { reg, .. }
                | RawRegVariable::MemR8 { reg, .. } => {
                    let tmp = self.allocator().borrow_mut().alloc_reg()?;
                    if reg == GpRegister::A {
                        self.cp(tmp.inner);
                    } else if tmp == GpRegister::A {
                        self.cp(reg);
                    }
                    self.jr(CpuFlag::Z.into(), imm);
                }
                _ => todo!()
            }
        }

        Ok(self)
    }
}