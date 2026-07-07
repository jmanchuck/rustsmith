use crate::rng::{RandGen, Rng, WeightedIndex};

use crate::generator::weights::EnumWeights;

#[derive(Clone, Copy)]
pub enum StmtVariants {
    LetStatement,
    ConditionalStatement,
    AssignStatement,
    LoopStatement,
    OpAssignStatement,
    FuncCallStatement,
}

impl StmtVariants {
    pub const ALL: &'static [Self] = &[
        Self::LetStatement,
        Self::ConditionalStatement,
        Self::AssignStatement,
        Self::LoopStatement,
        Self::OpAssignStatement,
        Self::FuncCallStatement,
    ];
}

impl RandGen for StmtVariants {
    fn rand_gen<R: Rng>(rng: &mut R) -> StmtVariants {
        let dist = WeightedIndex::new(StmtVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        StmtVariants::ALL[idx]
    }
}
