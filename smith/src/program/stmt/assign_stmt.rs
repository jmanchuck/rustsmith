use crate::program::{expr::expr::Expr, var::Var};

use super::stmt::Stmt;

pub struct AssignStmt {
    left_var: Var,
    right_expr: Expr,
}

impl AssignStmt {
    pub fn new(left_var: Var, right_expr: Expr) -> Self {
        AssignStmt {
            left_var,
            right_expr,
        }
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::AssignStatement(self)
    }
}

impl ToString for AssignStmt {
    fn to_string(&self) -> String {
        format!(
            "{} = {};",
            self.left_var.to_string(),
            self.right_expr.to_string()
        )
    }
}
