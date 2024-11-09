use super::Variable;

pub trait MetaInstructionTrait {
    fn var_ld(dest: Variable, src: Variable) -> Self;
    fn var_from_mem(dest: Variable, src: Variable) -> Self;
    fn var_to_mem(dest: Variable, src: Variable) -> Self;
    fn var_add(lhs: Variable, rhs: Variable) -> Self;
    fn var_inc(var: Variable) -> Self;
    fn var_sub(lhs: Variable, rhs: Variable) -> Self;
    fn var_dec(var: Variable) -> Self;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaInstruction {
    VarLd { dest: Variable, src: Variable },
    VarFromMem { dest: Variable, src: Variable },
    VarToMem { dest: Variable, src: Variable },
    VarAdd { lhs: Variable, rhs: Variable },
    VarInc { var: Variable },
    VarSub { lhs: Variable, rhs: Variable },
    VarDec { var: Variable },
}

impl MetaInstructionTrait for MetaInstruction {
    fn var_ld(dest: Variable, src: Variable) -> Self {
        Self::VarLd { dest, src }
    }

    fn var_from_mem(dest: Variable, src: Variable) -> Self {
        Self::VarFromMem { dest, src }
    }

    fn var_to_mem(dest: Variable, src: Variable) -> Self {
        Self::VarToMem { dest, src }
    }

    fn var_add(lhs: Variable, rhs: Variable) -> Self {
        Self::VarAdd { lhs, rhs }
    }

    fn var_inc(var: Variable) -> Self {
        Self::VarInc { var }
    }

    fn var_sub(lhs: Variable, rhs: Variable) -> Self {
        Self::VarSub { lhs, rhs }
    }

    fn var_dec(var: Variable) -> Self {
        Self::VarDec { var }
    }
}

