use crate::program::expr::expr::Expr;

use super::stmt::Stmt;

pub struct ExprStmt {
    expr: Expr,
}

impl ExprStmt {
    pub fn new(expr: Expr) -> Self {
        ExprStmt { expr }
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::ExprStatement(self)
    }
}

impl ToString for ExprStmt {
    fn to_string(&self) -> String {
        format!("{};", self.expr.to_string())
    }
}
