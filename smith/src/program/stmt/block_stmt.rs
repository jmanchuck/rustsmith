use std::collections::VecDeque;

use super::stmt::Stmt;

pub struct BlockStmt {
    stmts: VecDeque<Stmt>,
}

impl BlockStmt {
    pub fn new() -> Self {
        BlockStmt {
            stmts: VecDeque::new(),
        }
    }

    pub fn new_from_vec(stmts: Vec<Stmt>) -> Self {
        BlockStmt {
            stmts: VecDeque::from(stmts),
        }
    }

    pub fn len(&self) -> usize {
        self.stmts.len()
    }

    pub fn push(&mut self, stmt: Stmt) {
        self.stmts.push_back(stmt);
    }

    pub fn push_front(&mut self, stmt: Stmt) {
        self.stmts.push_front(stmt);
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        string.push_str("{\n");

        for stmt in self.stmts.iter() {
            string.push_str(&stmt.to_string()[..]);
            string.push('\n');
        }

        string.push('}');

        string
    }
}
