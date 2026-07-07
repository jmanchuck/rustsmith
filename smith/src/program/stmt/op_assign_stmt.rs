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
    deref: bool,
}

impl OpAssignStmt {
    pub fn new(left: Var, right: ArithmeticExpr, op: BinaryOp) -> Self {
        OpAssignStmt {
            left,
            right,
            op,
            deref: false,
        }
    }

    pub fn new_with_deref(left: Var, right: ArithmeticExpr, op: BinaryOp, deref: bool) -> Self {
        OpAssignStmt {
            left,
            right,
            op,
            deref,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.left.get_type()
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::OpAssignStatement(self)
    }
}

impl ToString for OpAssignStmt {
    // The place expression is duplicated on the right-hand side, which is
    // safe because place expressions have no side effects (unlike general
    // expressions, which may call functions with &mut parameters).
    fn to_string(&self) -> String {
        let deref = if self.deref { "*" } else { "" };
        let place = format!("{}{}", deref, self.left.to_string());
        let rhs = self.right.to_string();
        match self.op {
            BinaryOp::BITAND | BinaryOp::BITOR | BinaryOp::BITXOR => {
                format!("{} {}= {};", place, self.op.to_string(), rhs)
            }
            BinaryOp::DIV | BinaryOp::MOD => {
                let method = if let BinaryOp::DIV = self.op {
                    "checked_div"
                } else {
                    "checked_rem"
                };
                format!(
                    "{} = {{ let rhs = {}; ({}).{}(rhs).unwrap_or({}) }};",
                    place, rhs, place, method, place
                )
            }
            _ => format!(
                "{} = ({}).{}({});",
                place,
                place,
                self.op.to_string_safe(),
                rhs
            ),
        }
    }
}
