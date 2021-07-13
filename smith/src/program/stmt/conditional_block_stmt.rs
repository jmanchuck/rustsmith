#![allow(warnings)]
use crate::program::types::TypeID;

use super::{block_stmt::BlockStmt, conditional_stmt::ConditionalStmt};
use strum_macros::EnumDiscriminants;

// Use these on generation to enforce valid structure
#[derive(Clone, EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(ConditionalStmtVariants))]
pub enum ConditionalType {
    If,
    ElseIf,
    Else,
}

impl ToString for ConditionalType {
    fn to_string(&self) -> String {
        match self {
            ConditionalType::If => String::from("if"),
            ConditionalType::ElseIf => String::from("else if"),
            ConditionalType::Else => String::from("else"),
        }
    }
}

/* Conditional block is a block of conditional statements with constraints

The first statement in the block must be an IfStatement variant
Any subsequent statements must be ElseIfStatement or ElseStatement variant

IfStatement must be first
ElseIfStatement can be middle or last
ElseStatement can only be the last, cannot be middle

On generation:
- If can be followed by ElseIf or Else
- ElseIf can be followed by ElseIf or Else
- In expression form (returns a value) the last statement of each conditional must be expr_stmt/return_stmt
  and there must be an else statement
*/
pub struct ConditionalBlockStmt {
    block_stmt: BlockStmt,
    return_type: TypeID,
}

impl ConditionalBlockStmt {
    pub fn new(return_type: TypeID) -> Self {
        ConditionalBlockStmt {
            block_stmt: BlockStmt::new(),
            return_type,
        }
    }

    pub fn len(&self) -> usize {
        self.block_stmt.len()
    }

    pub fn insert(&mut self, conditional_stmt: ConditionalStmt) {
        // Sanity checks
        match conditional_stmt.get_conditional_type() {
            ConditionalType::If => {
                if self.len() != 0 {
                    panic!("Attempting to insert if statement in the middle of block conditional");
                }
            }
            ConditionalType::ElseIf | ConditionalType::Else => {
                if self.len() == 0 {
                    panic!("Attempting to insert else/else if statement at beginning of block conditional");
                }
            }
        }

        self.block_stmt.push(conditional_stmt.as_stmt());
    }
}
