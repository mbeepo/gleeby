use crate::memory::Addr;

use super::{variables::{Variable, RegSelector}, Id};

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
    fn alloc_reg(&mut self, kind: RegKind) -> Result<RegSelector, ConstAllocError> {
        
    }

    fn dealloc_reg(&mut self, reg: RegSelector) {
        self.registers.dealloc(reg)
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
    fn oversized_reg() -> Self {
        Self::TooBigForRegister
    }
}

impl Allocator<ConstAllocError> for ConstAllocator {
    fn alloc_reg(&mut self, var: Variable) -> Result<RegSelector, ConstAllocError> {
        self.registers.alloc(var.len.into())
    }

    fn dealloc_reg(&mut self, reg: RegSelector) {
        self.registers.dealloc(reg)
    }

    fn alloc_const(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.constants.alloc(len)
    }

    fn alloc_var(&mut self, len: u16) -> Result<Addr, ConstAllocError> {
        self.variables.alloc(len)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegKind {
    GpRegister,
    RegisterPair,
}

pub trait Allocator<AllocError>: std::fmt::Debug
        where AllocError: Clone + std::fmt::Debug + AllocErrorTrait {
    fn alloc_reg(&mut self, kind: RegKind) -> Result<RegSelector, AllocError>;
    fn dealloc_reg(&mut self, reg: RegSelector);
    fn alloc_const(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn dealloc_const(&mut self, addr: Addr);
    fn alloc_var(&mut self, len: u16) -> Result<Addr, AllocError>;
    fn dealloc_var(&mut self, addr: Addr);
}

pub trait AllocErrorTrait: Clone + std::fmt::Debug {
    fn oversized_reg() -> Self;
}