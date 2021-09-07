use crate::generator::weights::EnumWeights;

use super::variants::StmtVariants;

impl EnumWeights for StmtVariants {
    fn weight(&self) -> u32 {
        match self {
            StmtVariants::LetStatement => 2,
            StmtVariants::ConditionalStatement => 1,
            StmtVariants::AssignStatement => 3,
            StmtVariants::LoopStatement => 1,
            StmtVariants::OpAssignStatement => 2,
            StmtVariants::FuncCallStatement => 2,
        }
    }
}
