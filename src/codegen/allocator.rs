use std::{marker::PhantomData, ops::{Index, IndexMut}};

use crate::{cpu::{GpRegister, RegisterPair}, memory::Addr};

use super::{variables::RegSelector, Id};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct GpRegisters {
    pub a: Option<Id>,
    pub b: Option<Id>,
    pub c: Option<Id>,
    pub d: Option<Id>,
    pub e: Option<Id>,
    pub h: Option<Id>,
    pub l: Option<Id>,
    bleh: Option<Id>,
}

impl GpRegisters {
    fn alloc(&mut self) -> Result<GpRegister, ConstAllocError> {
        todo!()
    }

    fn dealloc(&mut self, reg: GpRegister) {
        let _ = reg;
        todo!()
    }

    fn alloc_pair(&mut self) -> Result<RegisterPair, ConstAllocError> {
        todo!()
    }

    fn dealloc_pair(&mut self, reg_pair: RegisterPair) {
        let _ = reg_pair;
        todo!()
    }

    fn claim(&mut self, reg: RegSelector, id: Id) {
        match reg {
            RegSelector::R8(r8) => self[r8] = Some(id),
            RegSelector::R16(r16) => {
                if let Ok((reg1, reg2)) = r16.try_split() {
                    self[reg1] = Some(id);
                    self[reg2] = Some(id);
                }
            }
        }
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
}

impl Index<GpRegister> for GpRegisters {
    type Output = Option<Id>;

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
    
            Ok(addr)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstAllocator {
    pub constants: AllocGroup,
    pub variables: AllocGroup,
    pub registers: GpRegisters,
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
    TooBigForRegister,
}

impl AllocErrorTrait for ConstAllocError {
    fn oversized_load() -> Self {
        Self::TooBigForRegister
    }
}

impl Allocator<ConstAllocError> for ConstAllocator {
    fn alloc_reg(&mut self) -> Result<GpRegister, ConstAllocError> {
        self.registers.alloc()
    }

    fn dealloc_reg(&mut self, reg: RegSelector) {
        self.registers.free(reg)
    }

    fn claim_reg(&mut self, reg: RegSelector, id: Id) {
        self.registers.claim(reg, id);
    }

    fn reg_free(&self, reg: RegSelector) -> bool {
        self.registers.is_claimed(reg)
    }

    fn alloc_reg_pair(&mut self) -> Result<RegisterPair, ConstAllocError> {
        self.registers.alloc_pair()
    }

    fn alloc_const(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.constants.alloc(len)
    }

    fn alloc_var(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.variables.alloc(len)
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
    fn alloc_reg(&mut self) -> Result<GpRegister, AllocError>;
    fn dealloc_reg(&mut self, reg: RegSelector);
    /// Claims a specific register/register pair for the given ID
    fn claim_reg(&mut self, reg: RegSelector, id: Id);
    /// Returns true if the selected register is unallocated
    fn reg_free(&self, reg: RegSelector) -> bool;
    fn alloc_reg_pair(&mut self) -> Result<RegisterPair, AllocError>;
    fn alloc_const(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn alloc_var(&mut self, len: u16) -> Result<Addr, AllocError>;
}

pub trait AllocErrorTrait: Clone + std::fmt::Debug {
    fn oversized_load() -> Self;
}