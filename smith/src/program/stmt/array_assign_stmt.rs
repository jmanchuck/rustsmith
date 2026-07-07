use crate::program::{
    expr::arithmetic_expr::{ArithmeticExpr, BinaryOp},
    types::ArrayTypeID,
};

use super::stmt::Stmt;

/// Guarded array element write: `arr[((idx) as usize) % LEN] = rhs;` plus the
/// op-assign wrapping forms. The `% LEN` guard makes any index expression
/// in-bounds (LEN is a nonzero literal), mirroring ArrayIndexExpr reads.
pub struct ArrayAssignStmt {
    array_name: String,
    array_type: ArrayTypeID,
    index: ArithmeticExpr,
    rhs: ArithmeticExpr,
    op: Option<BinaryOp>,
}

impl ArrayAssignStmt {
    pub fn new(
        array_name: String,
        array_type: ArrayTypeID,
        index: ArithmeticExpr,
        rhs: ArithmeticExpr,
        op: Option<BinaryOp>,
    ) -> Self {
        ArrayAssignStmt {
            array_name,
            array_type,
            index,
            rhs,
            op,
        }
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::ArrayAssignStatement(self)
    }
}

impl ToString for ArrayAssignStmt {
    // Plain writes keep the guarded index inline (each operand is evaluated
    // exactly once). Op-assign forms mention the place twice, and the index
    // expression may contain function calls with &mut side effects, so the
    // guarded index is bound to a local first — the place expression then has
    // no side effects, the same precondition OpAssignStmt relies on.
    fn to_string(&self) -> String {
        let guarded_index = format!(
            "(({}) as usize) % {}",
            self.index.to_string(),
            self.array_type.len
        );
        let rhs = self.rhs.to_string();

        let op = match self.op {
            None => {
                return format!("{}[{}] = {};", self.array_name, guarded_index, rhs);
            }
            Some(op) => op,
        };

        let place = format!("{}[idx]", self.array_name);
        let body = match op {
            BinaryOp::BITAND | BinaryOp::BITOR | BinaryOp::BITXOR => {
                format!("{} {}= {};", place, op.to_string(), rhs)
            }
            BinaryOp::DIV | BinaryOp::MOD => {
                let method = if let BinaryOp::DIV = op {
                    "checked_div"
                } else {
                    "checked_rem"
                };
                format!(
                    "let rhs = {}; {} = ({}).{}(rhs).unwrap_or({});",
                    rhs, place, place, method, place
                )
            }
            BinaryOp::SHL | BinaryOp::SHR => {
                // Same wrapping form as OpAssignStmt: plain <<= / >>= panics
                // under overflow-checks=on when the amount >= bit-width.
                format!(
                    "{} = ({}).{}(({}) as u32);",
                    place,
                    place,
                    op.to_string_safe(),
                    rhs
                )
            }
            _ => format!("{} = ({}).{}({});", place, place, op.to_string_safe(), rhs),
        };

        format!("{{ let idx = {}; {} }}", guarded_index, body)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::{expr::arithmetic_expr::IntExpr, types::IntTypeID};

    fn stmt(op: Option<BinaryOp>) -> ArrayAssignStmt {
        ArrayAssignStmt::new(
            "var_0".to_string(),
            ArrayTypeID::new(IntTypeID::I32, 4),
            IntExpr::new_u8(9).as_arith_expr(),
            IntExpr::new_i32(5).as_arith_expr(),
            op,
        )
    }

    #[test]
    fn plain_write_is_modulo_guarded() {
        assert_eq!(
            stmt(None).to_string(),
            "var_0[((9u8) as usize) % 4] = 5i32;"
        );
    }

    #[test]
    fn op_assign_write_binds_index_once() {
        assert_eq!(
            stmt(Some(BinaryOp::ADD)).to_string(),
            "{ let idx = ((9u8) as usize) % 4; var_0[idx] = (var_0[idx]).wrapping_add(5i32); }"
        );
    }

    #[test]
    fn div_op_assign_write_uses_checked_form() {
        assert_eq!(
            stmt(Some(BinaryOp::DIV)).to_string(),
            "{ let idx = ((9u8) as usize) % 4; let rhs = 5i32; \
             var_0[idx] = (var_0[idx]).checked_div(rhs).unwrap_or(var_0[idx]); }"
        );
    }

    #[test]
    fn shift_op_assign_write_uses_wrapping_form_with_u32_amount() {
        assert_eq!(
            stmt(Some(BinaryOp::SHL)).to_string(),
            "{ let idx = ((9u8) as usize) % 4; var_0[idx] = (var_0[idx]).wrapping_shl((5i32) as u32); }"
        );
    }

    #[test]
    fn bitwise_op_assign_write_uses_native_operator() {
        assert_eq!(
            stmt(Some(BinaryOp::BITXOR)).to_string(),
            "{ let idx = ((9u8) as usize) % 4; var_0[idx] ^= 5i32; }"
        );
    }
}
