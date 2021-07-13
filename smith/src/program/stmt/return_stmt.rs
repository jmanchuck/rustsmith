use crate::program::{expr::expr::Expr, types::TypeID};

use super::stmt::Stmt;

pub struct ReturnStmt {
    return_type: TypeID,
    expr: Expr,
    explicit_return: bool,
}

impl ReturnStmt {
    pub fn new(return_type: TypeID, expr: Expr) -> Self {
        ReturnStmt {
            return_type,
            expr,
            explicit_return: true,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.return_type.clone()
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::ReturnStatement(self)
    }
}

impl ToString for ReturnStmt {
    fn to_string(&self) -> String {
        if self.explicit_return {
            format!("return {};", self.expr.to_string())
        } else {
            self.expr.to_string()
        }
    }
}
