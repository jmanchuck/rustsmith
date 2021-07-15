use crate::program::expr::bool_expr::BoolExpr;

use super::{block_stmt::BlockStmt, stmt::Stmt};

pub struct ConditionalStmt {
    conditional_blocks: Vec<(BoolExpr, BlockStmt)>,
    else_body: Option<BlockStmt>,
}

impl ConditionalStmt {
    pub fn new() -> Self {
        ConditionalStmt {
            conditional_blocks: Vec::new(),
            else_body: None,
        }
    }

    pub fn new_from_vec(
        conditional_blocks: Vec<(BoolExpr, BlockStmt)>,
        else_body: Option<BlockStmt>,
    ) -> Self {
        ConditionalStmt {
            conditional_blocks,
            else_body,
        }
    }

    pub fn insert_conditional(&mut self, expr: BoolExpr, block_stmt: BlockStmt) {
        self.conditional_blocks.push((expr, block_stmt));
    }

    pub fn insert_else_body(&mut self, block_stmt: BlockStmt) {
        self.else_body = Some(block_stmt);
    }

    pub fn as_stmt(self) -> Stmt {
        Stmt::ConditionalStatement(self)
    }
}

impl ToString for ConditionalStmt {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for i in 0..self.conditional_blocks.len() {
            let conditional = if i == 0 { "if" } else { "else if" };

            result.push_str(
                format!(
                    "{} ({}) {}\n",
                    conditional,
                    self.conditional_blocks[i].0.to_string(),
                    self.conditional_blocks[i].1.to_string()
                )
                .as_str(),
            );
        }

        if let Some(else_block) = &self.else_body {
            result.push_str(format!("else {}\n", else_block.to_string()).as_str());
        }
        result
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
        let bool_expr_1 = BoolExpr::Bool(BoolValue::new(true));
        let bool_expr_2 = BoolExpr::Bool(BoolValue::new(false));

        let block_statements = vec![block_stmt(), block_stmt()];
        let conditions = vec![bool_expr_1, bool_expr_2];

        let conditional_blocks: Vec<(BoolExpr, BlockStmt)> = conditions
            .into_iter()
            .zip(block_statements.into_iter())
            .collect();

        let if_stmt = ConditionalStmt::new_from_vec(conditional_blocks, Some(block_stmt()));

        let string_rep = if_stmt.to_string();

        println!("{}", string_rep);

        assert!(string_rep.starts_with("if"));
        assert!(string_rep.contains("else if"));
        assert!(string_rep.contains("else"));
    }

    fn block_stmt() -> BlockStmt {
        let mut block_stmt = BlockStmt::new();

        let expr = IntExpr::new_u8(5).as_expr();
        let var = Var::new(IntTypeID::U8.as_type(), String::from("a"), false);
        let let_stmt = LetStmt::new(var, expr);

        block_stmt.push(let_stmt.as_stmt());

        block_stmt
    }
}
