use std::{cell::RefCell, marker::PhantomData, ops::{Index, IndexMut}, rc::Rc};

use crate::{cpu::{GpRegister, RegisterPair, SplitError}, memory::Addr};

use super::{variables::{MemoryVariable, RawRegVariable, RegSelector, RegVariable}, Id, Variable};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct GpRegisters {
    pub a: Option<(Id, Option<usize>)>,
    pub b: Option<(Id, Option<usize>)>,
    pub c: Option<(Id, Option<usize>)>,
    pub d: Option<(Id, Option<usize>)>,
    pub e: Option<(Id, Option<usize>)>,
    pub h: Option<(Id, Option<usize>)>,
    pub l: Option<(Id, Option<usize>)>,
    pub bleh: Option<(Id, Option<usize>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RcGpRegister {
    pub inner: GpRegister,
    pub(crate) allocator: Rc<RefCell<GpRegisters>>,
}

impl RcGpRegister {
    /// Strips reference count data from the register (trust me bro)
    /// 
    /// Calling this will prevent automatic register deallocation
    /// 
    /// It's up to you to make sure the register gets deallocated when its time comes
    pub fn into_raw(self) -> GpRegister {
        self.allocator.borrow_mut().release_rc(self.inner.into());
        self.inner
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RcRegisterPair {
    pub inner: RegisterPair,
    pub(crate) allocator: Rc<RefCell<GpRegisters>>,
}

impl RcRegisterPair {
    /// Strips reference count data from the register (trust me bro)
    /// 
    /// Calling this will prevent automatic register deallocation
    /// 
    /// It's up to you to make sure the register gets deallocated when its time comes
    pub fn into_raw(self) -> RegisterPair {
        self.allocator.borrow_mut().release_rc(self.inner.into());
        self.inner
    }
}

impl Clone for RcGpRegister {
    fn clone(&self) -> Self {
        self.allocator.borrow_mut().increment_rc(self.inner.into());

        Self {
            inner: self.inner,
            allocator: self.allocator.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RcRegVariable {
    pub inner: RawRegVariable,
    pub(crate) allocator: Rc<RefCell<GpRegisters>>
}

impl RcRegVariable {
    /// Strips reference count data from the register (trust me bro)
    /// 
    /// Calling this will prevent automatic register deallocation
    /// 
    /// It's up to you to make sure the register gets deallocated when its time comes
    pub fn into_raw(self) -> RawRegVariable {
        match self.inner {
            RawRegVariable::R8 { reg, .. }
            | RawRegVariable::MemR8 { reg, .. } => self.allocator.borrow_mut().release_rc(reg.into()),
            RawRegVariable::R16 { reg_pair, .. }
            | RawRegVariable::MemR16 { reg_pair, .. } => self.allocator.borrow_mut().release_rc(reg_pair.into()),
            _ => {}
        }
        self.inner
    }
}

impl Clone for RcRegVariable {
    fn clone(&self) -> Self {
        let reg: Option<RegSelector> = match self.inner {
            RawRegVariable::R8 { reg, .. }
            | RawRegVariable::MemR8 { reg, .. } => Some(reg.into()),
            RawRegVariable::R16 { reg_pair, .. }
            | RawRegVariable::MemR16 { reg_pair, .. } => Some(reg_pair.into()),
            RawRegVariable::UnallocatedR8(..)
            | RawRegVariable::UnallocatedR16(..) => None,
        };

        if let Some(reg) = reg {
            self.allocator.borrow_mut().increment_rc(reg);
        }

        Self {
            inner: self.inner,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for RcRegVariable {
    fn drop(&mut self) {
        println!("Drop: {:#?}", self.inner);
        match self.inner {
            RawRegVariable::R8 { reg, .. }
            | RawRegVariable::MemR8 { reg, .. } => self.allocator.borrow_mut().decrement_rc(reg.into()),
            RawRegVariable::R16 { reg_pair, .. }
            | RawRegVariable::MemR16 { reg_pair, .. } => self.allocator.borrow_mut().decrement_rc(reg_pair.into()),
            _ => {},
        };
    }
}

impl Drop for RcGpRegister {
    fn drop(&mut self) {
        println!("Drop: {:#?}", self.inner);
        self.allocator.borrow_mut().decrement_rc(self.inner.into());
    }
}

impl Drop for RcRegisterPair {
    fn drop(&mut self) {
        println!("Drop: {:#?}", self.inner);
        self.allocator.borrow_mut().decrement_rc(self.inner.into());
    }
}

impl PartialEq<RcGpRegister> for GpRegister {
    fn eq(&self, other: &RcGpRegister) -> bool {
        self.eq(&other.inner)
    }
}

impl PartialEq<GpRegister> for RcGpRegister {
    fn eq(&self, other: &GpRegister) -> bool {
        other.eq(&self.inner)
    }
}

impl PartialEq<RcRegisterPair> for RegisterPair {
    fn eq(&self, other: &RcRegisterPair) -> bool {
        self.eq(&other.inner)
    }
}

impl PartialEq<RegisterPair> for RcRegisterPair {
    fn eq(&self, other: &RegisterPair) -> bool {
        other.eq(&self.inner)
    }
}

impl From<&RcGpRegister> for GpRegister {
    fn from(value: &RcGpRegister) -> Self {
        value.inner
    }
}

impl From<&RcRegisterPair> for RegisterPair {
    fn from(value: &RcRegisterPair) -> Self {
        value.inner
    }
}

impl RcRegisterPair {
    pub fn try_split(&self) -> Result<(GpRegister, GpRegister), SplitError> {
        use RegisterPair::*;

        let (reg1, reg2) = match self.inner {
            BC => {
                (GpRegister::B, GpRegister::C)
            },
            DE => {
                (GpRegister::D, GpRegister::E)
            },
            HL => {
                (GpRegister::H, GpRegister::L)
            },
            SP => Err(SplitError::StackPointer)?,
        };

        Ok((reg1, reg2))
    }
}

impl GpRegisters {
    fn alloc(&mut self) -> Result<GpRegister, ConstAllocError> {
        let inner = if self.a == None {
            self.a = Some((Id::Unset, Some(1)));
            GpRegister::A
        } else if self.b == None {
            self.b = Some((Id::Unset, Some(1)));
            GpRegister::B
        } else if self.c == None {
            self.c = Some((Id::Unset, Some(1)));
            GpRegister::C
        } else if self.d == None {
            self.d = Some((Id::Unset, Some(1)));
            GpRegister::D
        } else if self.e == None {
            self.e = Some((Id::Unset, Some(1)));
            GpRegister::E
        } else if self.h == None {
            self.h = Some((Id::Unset, Some(1)));
            GpRegister::H
        } else if self.l == None {
            self.l = Some((Id::Unset, Some(1)));
            GpRegister::L
        } else {
            Err(ConstAllocError::OutOfRegisters)?
        };

        dbg!(inner);

        Ok(inner)
    }

    fn alloc_pair(&mut self) -> Result<RegisterPair, ConstAllocError> {
        let inner = if self.b == None && self.c == None {
            self.b = Some((Id::Unset, Some(1)));
            self.c = Some((Id::Unset, Some(1)));
            RegisterPair::BC
        } else if self.d == None && self.e == None {
            self.d = Some((Id::Unset, Some(1)));
            self.e = Some((Id::Unset, Some(1)));
            RegisterPair::DE
        } else if self.h == None && self.l == None {
            self.h = Some((Id::Unset, Some(1)));
            self.l = Some((Id::Unset, Some(1)));
            RegisterPair::HL
        } else {
            Err(ConstAllocError::OutOfRegisters)?
        };
        
        Ok(inner)
    }

    fn claim(&mut self, reg: GpRegister, id: Id) -> GpRegister {
        self[reg] = Some((id, Some(1)));
        reg
    }

    fn claim_pair(&mut self, reg_pair: RegisterPair, id: Id) -> RegisterPair {
        if let Ok((reg1, reg2)) = reg_pair.try_split() {
            self[reg1] = Some((id, Some(1)));
            self[reg2] = Some((id, Some(1)));
        }
        
        reg_pair
    }

    fn free(&mut self, reg: RegSelector) {
        match reg {
            RegSelector::R8(r8) => self[r8] = None,
            RegSelector::R16(r16) => {
                if let Ok((reg1, reg2)) = r16.try_split() {
                    self[reg1] = None;
                    self[reg2] = None;
                }
            }
        }
    }

    fn is_claimed(&self, reg: RegSelector) -> bool {
        match reg {
            RegSelector::R8(r8) => self[r8] != None,
            RegSelector::R16(r16) => {
                if let Ok((reg1, reg2)) = r16.try_split() {
                    self[reg1] != None && self[reg2] != None
                } else {
                    false
                }
            }
        }
    }

    pub(crate) fn increment_rc(&mut self, reg: RegSelector) {
        self.crement_rc(reg, Crementivity::Up);
    }

    pub(crate) fn decrement_rc(&mut self, reg: RegSelector) {
        self.crement_rc(reg, Crementivity::Down);
    }

    fn crement_rc(&mut self, reg: RegSelector, way: Crementivity) {
        let by = match way {
            Crementivity::Up => 1,
            Crementivity::Down => -1,
        };

        match reg {
            RegSelector::R8(r8) => {
                self[r8] = self[r8].and_then(|(id, rc)| Some((id, rc.and_then(|rc| Some(rc.checked_add_signed(by).expect("That's a lotta cloning"))))));
                if self[r8].is_some_and(|(_, rc)| rc == Some(0)) {
                    self.free(r8.into());
                }
            }
            RegSelector::R16(r16) => {
                if let Ok((reg1, reg2)) = r16.try_split() {
                    self[reg1] = self[reg1].and_then(|(id, rc)| Some((id, rc.and_then(|rc| Some(rc.checked_add_signed(by).expect("That's a lotta cloning"))))));
                    self[reg2] = self[reg2].and_then(|(id, rc)| Some((id, rc.and_then(|rc| Some(rc.checked_add_signed(by).expect("That's a lotta cloning"))))));
                    if self[reg1].is_some_and(|(_, rc)| rc == Some(0)) {
                        self.free(reg1.into());
                    }
                    if self[reg2].is_some_and(|(_, rc)| rc == Some(0)) {
                        self.free(reg2.into());
                    }
                } else {
                    unreachable!("SP is not reference counted");
                }
            }
        }
    }

    pub(crate) fn release_rc(&mut self, reg: RegSelector) {
        match reg {
            RegSelector::R8(r8) => self[r8] = self[r8].map(|(id, _)| (id, None)),
            RegSelector::R16(r16) => {
                if let Ok((reg1, reg2)) = r16.try_split() {
                    self[reg1] = self[reg1].map(|(id, _)| (id, None));
                    self[reg2] = self[reg1].map(|(id, _)| (id, None));
                }
            }
        }
    }
}

enum Crementivity { Up, Down }

impl Index<GpRegister> for GpRegisters {
    type Output = Option<(Id, Option<usize>)>;

    fn index(&self, index: GpRegister) -> &Self::Output {
        use GpRegister::*;
        match index {
            A => &self.a,
            B => &self.b,
            C => &self.c,
            D => &self.d,
            E => &self.e,
            H => &self.h,
            L => &self.l,
            IndHL => &None,
        }
    }
}

impl IndexMut<GpRegister> for GpRegisters {
    fn index_mut(&mut self, index: GpRegister) -> &mut Self::Output {
        use GpRegister::*;
        match index {
            A => &mut self.a,
            B => &mut self.b,
            C => &mut self.c,
            D => &mut self.d,
            E => &mut self.e,
            H => &mut self.h,
            L => &mut self.l,
            IndHL => &mut self.bleh,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocGroup {
    pub next: Addr,
    pub offset: Addr,
    pub len: Addr,
    pub used: u16,
}

impl AllocGroup {
    pub fn alloc(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        let addr = self.next;

        if addr + len as u16 > self.len {
            Err(ConstAllocError::OutOfMemory)
        } else {
            self.next += len as u16;
    
            Ok(addr + self.offset)
        }
    }

    pub fn dealloc(&mut self, addr: Addr) -> Result<(), ConstAllocError> {
        // bumpy
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstAllocator {
    pub constants: AllocGroup,
    pub variables: AllocGroup,
    pub registers: Rc<RefCell<GpRegisters>>,
}

impl Default for ConstAllocator {
    fn default() -> Self {
        let constants = AllocGroup {
            next: 0,
            offset: 0x0000,
            len: 0x0800,
            used: 0,
        };
        let variables = AllocGroup {
            next: 0,
            offset: 0x0000,
            len: 0x1000,
            used: 0,
        };

        Self {
            constants,
            variables,
            registers: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstAllocError {
    OutOfMemory,
    OutOfRegisters,
    TooBigForRegister,
}

impl AllocErrorTrait for ConstAllocError {
    fn oversized_load() -> Self {
        Self::TooBigForRegister
    }
}

impl Allocator<ConstAllocError> for ConstAllocator {
    fn alloc_reg(&mut self) -> Result<RcGpRegister, ConstAllocError> {
        Ok(RcGpRegister {
            inner: self.registers.borrow_mut().alloc()?,
            allocator: self.registers.clone(),
        })
    }

    fn alloc_reg_pair(&mut self) -> Result<RcRegisterPair, ConstAllocError> {
        Ok(RcRegisterPair {
            inner: self.registers.borrow_mut().alloc_pair()?,
            allocator: self.registers.clone(),
        })
    }

    fn release_reg(&mut self, reg: RegSelector) -> &mut Self {
        self.registers.borrow_mut().free(reg);
        self
    }

    fn claim_reg(&mut self, reg: GpRegister, id: Id) -> RcGpRegister {
        RcGpRegister {
            inner: self.registers.borrow_mut().claim(reg, id),
            allocator: self.registers.clone(),
        }
    }

    fn claim_reg_pair(&mut self, reg_pair: RegisterPair, id: Id) -> RcRegisterPair {
        RcRegisterPair {
            inner: self.registers.borrow_mut().claim_pair(reg_pair, id),
            allocator: self.registers.clone(),
        }
    }

    fn reg_is_used(&self, reg: RegSelector) -> bool {
        self.registers.borrow_mut().is_claimed(reg)
    }

    fn alloc_const(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.constants.alloc(len)
    }

    fn alloc_var(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.variables.alloc(len)
    }

    fn dealloc_var(&mut self, var: Variable) -> Result<&mut Self, ConstAllocError> {
        match var {
            Variable::Memory(MemoryVariable { addr, .. }) => { self.variables.dealloc(addr)?; },
            Variable::Reg(var) => match var {
                RegVariable::Rc(var) => match var.inner {
                    RawRegVariable::R8 { reg, .. } => { self.registers.borrow_mut().decrement_rc(reg.clone().into()); },
                    RawRegVariable::R16 { reg_pair, .. } => { self.registers.borrow_mut().decrement_rc(reg_pair.clone().into()); },
                    RawRegVariable::MemR8 { addr, reg, .. } => {
                        self.registers.borrow_mut().decrement_rc(reg.clone().into());
                        self.variables.dealloc(addr)?;
                    }
                    RawRegVariable::MemR16 { addr, reg_pair, .. } => {
                        self.registers.borrow_mut().decrement_rc(reg_pair.clone().into());
                        self.variables.dealloc(addr)?;
                    }
                    _ => {}
                },
                RegVariable::Raw(var) => match var {
                    RawRegVariable::R8 { reg, .. } => { self.registers.borrow_mut().free(reg.into()); }
                    RawRegVariable::R16 { reg_pair, .. } => { self.registers.borrow_mut().free(reg_pair.into()); } 
                    RawRegVariable::MemR8 { addr, reg, .. } => {
                        self.registers.borrow_mut().free(reg.into());
                        self.variables.dealloc(addr)?;
                    }
                    RawRegVariable::MemR16 { addr, reg_pair, .. } => {
                        self.registers.borrow_mut().free(reg_pair.into());
                        self.variables.dealloc(addr)?;
                    }
                    RawRegVariable::UnallocatedR8(_)
                    | RawRegVariable::UnallocatedR16(_) => {}
                }
            }
            _ => {}
        };

        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegKind<AllocError>
        where AllocError: Clone + std::fmt::Debug + AllocErrorTrait {
    GpRegister,
    RegisterPair,
    OkTypecheckerYouWin(PhantomData<AllocError>),
}

impl<AllocError> RegKind<AllocError>
        where AllocError: Clone + std::fmt::Debug + AllocErrorTrait {
    pub fn try_from_len(len: u16) -> Result<Self, AllocError> {
        if len == 1 { Ok(Self::GpRegister) }
        else if len == 2 { Ok(Self::RegisterPair) }
        else { Err(AllocError::oversized_load())}
    }
}

pub trait Allocator<AllocError>: std::fmt::Debug
        where AllocError: Clone + std::fmt::Debug + AllocErrorTrait {
    fn alloc_reg(&mut self) -> Result<RcGpRegister, AllocError>;
    fn alloc_reg_pair(&mut self) -> Result<RcRegisterPair, AllocError>;
    fn release_reg(&mut self, reg: RegSelector) -> &mut Self;
    /// Claims a specific register for the given ID
    fn claim_reg(&mut self, reg: GpRegister, id: Id) -> RcGpRegister;
    /// Claims a specific register pair for the given ID
    fn claim_reg_pair(&mut self, reg: RegisterPair, id: Id) -> RcRegisterPair;
    /// Returns true if the selected register is unallocated
    fn reg_is_used(&self, reg: RegSelector) -> bool;
    fn alloc_const(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn alloc_var(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn dealloc_var(&mut self, var: Variable) -> Result<&mut Self, AllocError>;
}

pub trait AllocErrorTrait: Clone + std::fmt::Debug {
    fn oversized_load() -> Self;
}