use super::stmt::Stmt;
use crate::program::expr::expr::Expr;
use crate::program::types::TypeID;

pub struct StaticStmt {
    var_name: String,
    var_type: TypeID,
    expr: Expr,
}

impl StaticStmt {
    pub fn new(var_name: String, var_type: TypeID, expr: Expr) -> Self {
        StaticStmt {
            var_name,
            var_type,
            expr,
        }
    }
}

impl StaticStmt {
    pub fn to_string(&self) -> String {
        format!(
            "static {}: {} = {};",
            self.var_name,
            self.var_type.to_string(),
            self.expr.to_string()
        )
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::StaticStatement(self)
    }
}
