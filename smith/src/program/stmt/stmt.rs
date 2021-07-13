use super::{
    assign_stmt::AssignStmt, conditional_stmt::ConditionalStmt, expr_stmt::ExprStmt,
    let_stmt::LetStmt, return_stmt::ReturnStmt, static_stmt::StaticStmt,
};
use strum_macros::{EnumCount, EnumDiscriminants, EnumIter};

#[derive(EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(StmtVariants))]
#[strum_discriminants(derive(EnumCount, EnumIter))]
pub enum Stmt {
    LetStatement(LetStmt),
    StaticStatement(StaticStmt),
    ConditionalStatement(ConditionalStmt),
    AssignStatement(AssignStmt),
    ReturnStatement(ReturnStmt),
    ExprStatement(ExprStmt),
}

impl ToString for Stmt {
    fn to_string(&self) -> String {
        match self {
            Self::LetStatement(s) => s.to_string(),
            Self::StaticStatement(s) => s.to_string(),
            Self::ConditionalStatement(s) => s.to_string(),
            Self::AssignStatement(s) => s.to_string(),
            Self::ReturnStatement(s) => s.to_string(),
            Self::ExprStatement(s) => s.to_string(),
        }
    }
}
