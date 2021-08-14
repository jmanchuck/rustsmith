use crate::program::types::BorrowTypeID;

use super::expr::Expr;
// The explicit field refers to whether the variable in scope is already a borrow type
// i.e. if variable &mut a is passed as mutable reference, we don't pass using
// function(&mut a) and instead directly do function(a)
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
                "{}{}",
                self.borrow_type_id.to_string(),
                self.expr.to_string()
            )
        } else {
            self.expr.to_string()
        }
    }
}

impl std::fmt::Debug for BorrowExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorrowExpr")
            .field("borrow_type", &self.borrow_type_id.to_string())
            .field("expr", &self.expr.to_string())
            .field("explicit", &self.explicit)
            .finish()
    }
}
