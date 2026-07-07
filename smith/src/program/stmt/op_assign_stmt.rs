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
            BinaryOp::SHL | BinaryOp::SHR => {
                // Same wrapping form as BinaryExpr: plain <<= / >>= panics
                // under overflow-checks=on when the amount >= bit-width, so
                // the compound-assign operator form must not be used.
                format!(
                    "{} = ({}).{}(({}) as u32);",
                    place,
                    place,
                    self.op.to_string_safe(),
                    rhs
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::{
        expr::arithmetic_expr::IntExpr,
        types::{IntTypeID, TypeID},
    };

    #[test]
    fn shift_op_assign_uses_wrapping_form_with_u32_amount() {
        let var = Var::new(
            TypeID::IntType(IntTypeID::I32),
            String::from("a"),
            false,
        );
        let stmt = OpAssignStmt::new(var, IntExpr::new_u8(3).as_arith_expr(), BinaryOp::SHL);
        assert_eq!(stmt.to_string(), "a = (a).wrapping_shl((3u8) as u32);");

        let var = Var::new(
            TypeID::IntType(IntTypeID::U64),
            String::from("b"),
            false,
        );
        let stmt = OpAssignStmt::new(var, IntExpr::new_i32(2).as_arith_expr(), BinaryOp::SHR);
        assert_eq!(stmt.to_string(), "b = (b).wrapping_shr((2i32) as u32);");
    }
}
