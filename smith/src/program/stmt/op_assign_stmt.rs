use crate::program::{
    expr::arithmetic_expr::{ArithmeticExpr, BinaryOp},
    types::TypeID,
    var::Var,
};

use super::stmt::Stmt;

pub struct OpAssignStmt {
    left: Var,
    right: ArithmeticExpr,
    op: BinaryOp,
}

impl OpAssignStmt {
    pub fn new(left: Var, right: ArithmeticExpr, op: BinaryOp) -> Self {
        OpAssignStmt { left, right, op }
    }

    pub fn get_type(&self) -> TypeID {
        self.left.get_type()
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::OpAssignStatement(self)
    }

    pub fn to_string_safe(&self) -> String {
        format!(
            "{}.{}({});",
            self.left.to_string(),
            self.op.to_string_self_safe(),
            self.right.to_string()
        )
    }
}

impl ToString for OpAssignStmt {
    fn to_string(&self) -> String {
        self.to_string_safe()
    }
}
