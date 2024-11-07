use crate::memory::Addr;

use super::{variables::{ConfirmedVariable, MemoryVariable, RegVariable}, Ctx, Id};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct GpRegisters {
    pub a: Option<(Id, Ctx)>,
    pub b: Option<(Id, Ctx)>,
    pub c: Option<(Id, Ctx)>,
    pub d: Option<(Id, Ctx)>,
    pub e: Option<(Id, Ctx)>,
    pub h: Option<(Id, Ctx)>,
    pub l: Option<(Id, Ctx)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocGroup {
    pub next: Addr,
    pub offset: Addr,
    pub len: Addr,
    pub used: u16,
}

impl AllocGroup {
    pub fn alloc(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        let len = data.len();
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstAllocError {
    OutOfMemory,
    TooBigForRegister,
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

impl Allocator<ConstAllocError> for ConstAllocator {
    fn new_const(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        self.constants.alloc(data)
    }

    fn new_var(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        self.variables.alloc(data)
    }

    fn alloc_reg(&mut self, var: ConfirmedVariable) -> Result<RegVariable, ConstAllocError> {
        todo!()
    }
}

pub trait Allocator<Error>: std::fmt::Debug
        where Error: Clone + std::fmt::Debug {
    fn new_const(&mut self, data: &[u8]) -> Result<Addr, Error>;
    fn new_var(&mut self, data: &[u8]) -> Result<Addr, Error>;
    fn alloc_reg(&mut self, var: ConfirmedVariable) -> Result<RegVariable, Error>;
}