use super::stmt::Stmt;
use crate::program::{expr::expr::Expr, var::Var};

pub struct LetStmt {
    var: Var,
    expr: Expr,
}

impl LetStmt {
    pub fn new(var: Var, expr: Expr) -> Self {
        LetStmt { var, expr }
    }

    pub fn is_mut(&self) -> bool {
        self.var.is_mut()
    }
}

impl LetStmt {
    pub fn to_string(&self) -> String {
        let mut_qualifier = if self.is_mut() {
            String::from(" mut")
        } else {
            String::new()
        };

        format!(
            "let{} {}: {}{} = {};",
            mut_qualifier,
            self.var.get_name(),
            self.var.get_borrow_type().to_string(),
            self.var.get_type().to_string(),
            self.expr.to_string()
        )
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::LetStatement(self)
    }
}

#[cfg(test)]
mod test {
    use crate::program::{
        expr::arithmetic_expr::{ArithmeticExpr, IntExpr},
        types::{IntTypeID, TypeID},
    };

    use super::*;
    #[test]
    fn has_correct_string_representation() {
        let var_type = TypeID::IntType(IntTypeID::I32);
        let var = Var::new(var_type, String::from("a"), false);

        // Const integer expression
        let expr = IntExpr::new_i32(10);
        let expr = Expr::Arithmetic(ArithmeticExpr::Int(expr));
        let let_stmt = LetStmt::new(var, expr);

        assert_eq!(let_stmt.to_string(), "let a: i32 = 10i32;".to_string());
    }
}
