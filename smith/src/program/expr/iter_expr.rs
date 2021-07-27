use crate::program::types::{IntTypeID, TypeID};

use super::arithmetic_expr::ArithmeticExpr;

pub enum IterExpr {
    Range(IterRange),
}

impl ToString for IterExpr {
    fn to_string(&self) -> String {
        match self {
            IterExpr::Range(s) => s.to_string(),
        }
    }
}

pub struct IterRange {
    type_id: TypeID,
    left: ArithmeticExpr,
    right: ArithmeticExpr,
}

impl IterRange {
    pub fn new(int_type_id: IntTypeID, left: ArithmeticExpr, right: ArithmeticExpr) -> Self {
        IterRange {
            type_id: int_type_id.as_type(),
            left,
            right,
        }
    }

    pub fn as_iter_expr(self) -> IterExpr {
        IterExpr::Range(self)
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }
}

impl ToString for IterRange {
    fn to_string(&self) -> String {
        format!("{}..{}", self.left.to_string(), self.right.to_string())
    }
}
