#![allow(warnings)]
use super::expr::Expr;
use crate::program::types::TypeID;

pub struct RefCellExpr {
    expr: Expr,
    type_id: TypeID,
}

impl RefCellExpr {
    pub fn new(expr: Expr, type_id: TypeID) -> Self {
        RefCellExpr { expr, type_id }
    }
}

impl ToString for RefCellExpr {
    fn to_string(&self) -> String {
        format!("RefCell::new({})", self.expr.to_string())
    }
}
