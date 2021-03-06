use super::{
    assign_stmt::AssignStmt, conditional_stmt::ConditionalStmt, expr_stmt::ExprStmt,
    for_loop_stmt::ForLoopStmt, let_stmt::LetStmt, op_assign_stmt::OpAssignStmt,
    return_stmt::ReturnStmt, static_stmt::StaticStmt,
};
pub enum Stmt {
    LetStatement(LetStmt),
    StaticStatement(StaticStmt),
    ConditionalStatement(ConditionalStmt),
    AssignStatement(AssignStmt),
    ReturnStatement(ReturnStmt),
    ExprStatement(ExprStmt),
    LoopStatement(ForLoopStmt),
    OpAssignStatement(OpAssignStmt),
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
            Self::LoopStatement(s) => s.to_string(),
            Self::OpAssignStatement(s) => s.to_string(),
        }
    }
}
