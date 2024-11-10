use super::{variables::Constant, Variable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarOrConst {
    Var(Variable),
    Const(Constant),
}

pub trait MetaInstructionTrait {
    fn set_var(dest: Variable, src: VarOrConst) -> Self;
    fn var_from_ind(dest: Variable, src: Variable) -> Self;
    fn var_to_ind(dest: Variable, src: Variable) -> Self;
    fn add_var(lhs: Variable, rhs: Variable) -> Self;
    fn inc_var(var: Variable) -> Self;
    fn sub_var(lhs: Variable, rhs: Variable) -> Self;
    fn dec_var(var: Variable) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaInstruction {
    VarSet { dest: Variable, src: VarOrConst },
    VarFromInd { dest: Variable, src: Variable },
    VarToInd { dest: Variable, src: Variable },
    VarAdd { lhs: Variable, rhs: Variable },
    VarInc { var: Variable },
    VarSub { lhs: Variable, rhs: Variable },
    VarDec { var: Variable },
}

impl MetaInstructionTrait for MetaInstruction {
    fn set_var(dest: Variable, src: VarOrConst) -> Self {
        Self::VarSet { dest, src }
    }

    fn var_from_ind(dest: Variable, src: Variable) -> Self {
        Self::VarFromInd { dest, src }
    }

    fn var_to_ind(dest: Variable, src: Variable) -> Self {
        Self::VarToInd { dest, src }
    }

    fn add_var(lhs: Variable, rhs: Variable) -> Self {
        Self::VarAdd { lhs, rhs }
    }

    fn inc_var(var: Variable) -> Self {
        Self::VarInc { var }
    }

    fn sub_var(lhs: Variable, rhs: Variable) -> Self {
        Self::VarSub { lhs, rhs }
    }

    fn dec_var(var: Variable) -> Self {
        Self::VarDec { var }
    }
}

