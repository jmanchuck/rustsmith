use crate::program::types::BorrowTypeID;

use super::expr::Expr;

pub struct BorrowExpr {
    borrow_type_id: BorrowTypeID,
    expr: Expr,
    explicit: bool,
}

impl BorrowExpr {
    pub fn new(borrow_type_id: BorrowTypeID, expr: Expr, explicit: bool) -> Self {
        BorrowExpr {
            borrow_type_id,
            expr,
            explicit,
        }
    }

    pub fn as_expr(self) -> Expr {
        Expr::Borrow(Box::new(self))
    }
}

impl ToString for BorrowExpr {
    fn to_string(&self) -> String {
        if self.explicit {
            format!(
                "{}({})",
                self.borrow_type_id.to_string(),
                self.expr.to_string()
            )
        } else {
            self.expr.to_string()
        }
    }
}
