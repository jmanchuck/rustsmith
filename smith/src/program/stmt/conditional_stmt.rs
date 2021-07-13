use crate::program::expr::bool_expr::BoolExpr;

use super::{block_stmt::BlockStmt, conditional_block_stmt::ConditionalType, stmt::Stmt};

pub struct ConditionalStmt {
    condition: BoolExpr,
    block_stmt: BlockStmt,
    conditional_type: ConditionalType,
}

impl ConditionalStmt {
    pub fn new(
        condition: BoolExpr,
        block_stmt: BlockStmt,
        conditional_type: ConditionalType,
    ) -> Self {
        ConditionalStmt {
            condition,
            block_stmt,
            conditional_type,
        }
    }

    pub fn get_conditional_type(&self) -> ConditionalType {
        self.conditional_type.clone()
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::ConditionalStatement(self)
    }
}

impl ToString for ConditionalStmt {
    fn to_string(&self) -> String {
        match self.conditional_type {
            ConditionalType::Else => {
                format!("else {}", self.block_stmt.to_string())
            }

            _ => {
                let mut string = String::new();
                string.push_str(
                    format!(
                        "{} {} {}\n",
                        self.conditional_type.to_string(),
                        self.condition.to_string(),
                        self.block_stmt.to_string()
                    )
                    .as_str(),
                );
                string
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::program::{
        expr::{bool_expr::BoolValue, int_expr::IntExpr},
        stmt::let_stmt::LetStmt,
        types::IntTypeID,
        var::Var,
    };

    use super::*;
    #[test]
    fn correct_structure() {
        let bool_expr = BoolExpr::Bool(BoolValue::new(true));
        let mut block_stmt = BlockStmt::new();

        let expr = IntExpr::new_u8(5).as_expr();
        let var = Var::new(IntTypeID::U8.as_type(), String::from("a"), false);
        let let_stmt = LetStmt::new(var, expr);

        block_stmt.push(let_stmt.as_stmt());

        let if_stmt = ConditionalStmt::new(bool_expr, block_stmt, ConditionalType::If);

        let string_rep = if_stmt.to_string();

        println!("{}", string_rep);

        assert!(string_rep.starts_with("if"));
    }
}
