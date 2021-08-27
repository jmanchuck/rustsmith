use crate::program::{
    expr::iter_expr::IterExpr,
    stmt::{block_stmt::BlockStmt, stmt::Stmt},
    types::TypeID,
    var::Var,
};

// TypeID is for the loop's iter expression
pub struct ForLoopStmt {
    type_id: TypeID,
    iter_var: Var,
    iterable: IterExpr,
    block_stmt: BlockStmt,
}

impl ForLoopStmt {
    pub fn new(type_id: TypeID, iter_var: Var, iterable: IterExpr, block_stmt: BlockStmt) -> Self {
        ForLoopStmt {
            type_id,
            iter_var,
            iterable,
            block_stmt,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::LoopStatement(self)
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.block_stmt.push(stmt);
    }
}

impl ToString for ForLoopStmt {
    fn to_string(&self) -> String {
        format!(
            "for {} in {} {}",
            self.iter_var.get_name(),
            self.iterable.to_string(),
            self.block_stmt.to_string()
        )
    }
}

#[cfg(test)]
mod test {
    use crate::program::{
        expr::{arithmetic_expr::IntExpr, iter_expr::IterRange},
        types::IntTypeID,
    };

    use super::*;

    #[test]
    fn for_loop_correct_string_representation() {
        let lower_range = IntExpr::new_i8(2).as_arith_expr();
        let upper_range = IntExpr::new_i8(8).as_arith_expr();
        let iter_expr = IterRange::new(IntTypeID::I8, lower_range, upper_range).as_iter_expr();

        let var = Var::new(IntTypeID::I8.as_type(), "i".to_string(), false);

        let block_stmt = BlockStmt::new();

        let for_loop_expr = ForLoopStmt::new(IntTypeID::I8.as_type(), var, iter_expr, block_stmt);

        println!("{}", for_loop_expr.to_string());

        assert!(for_loop_expr.to_string().contains("for i in 2i8..8i8"));
    }
}
