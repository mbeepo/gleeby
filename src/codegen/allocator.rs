use std::marker::PhantomData;

use crate::{cpu::{GpRegister, RegisterPair}, memory::Addr};

use super::Id;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct GpRegisters {
    pub a: Option<Id>,
    pub b: Option<Id>,
    pub c: Option<Id>,
    pub d: Option<Id>,
    pub e: Option<Id>,
    pub h: Option<Id>,
    pub l: Option<Id>,
}

impl GpRegisters {
    fn alloc(&mut self) -> Result<GpRegister, ConstAllocError> {
        todo!()
    }

    fn dealloc(&mut self, reg: GpRegister) {
        todo!()
    }

    fn alloc_pair(&mut self) -> Result<RegisterPair, ConstAllocError> {
        todo!()
    }

    fn dealloc_pair(&mut self, reg_pair: RegisterPair) {
        todo!()
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

    fn dealloc_reg(&mut self, reg: GpRegister) {
        self.registers.dealloc(reg)
    }

    fn alloc_reg_pair(&mut self) -> Result<RegisterPair, ConstAllocError> {
        self.registers.alloc_pair()
    }

    fn dealloc_reg_pair(&mut self, reg_pair: RegisterPair) {
        self.registers.dealloc_pair(reg_pair);
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
    fn dealloc_reg(&mut self, reg: GpRegister);
    fn alloc_reg_pair(&mut self) -> Result<RegisterPair, AllocError>;
    fn dealloc_reg_pair(&mut self, reg_pair: RegisterPair);
    fn alloc_const(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn alloc_var(&mut self, len: u16) -> Result<Addr, AllocError>;
}

pub trait AllocErrorTrait: Clone + std::fmt::Debug {
    fn oversized_load() -> Self;
}