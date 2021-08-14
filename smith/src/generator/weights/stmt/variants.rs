use rand::{
    distributions::{Distribution, Standard, WeightedIndex},
    Rng,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter};

use crate::generator::weights::EnumWeights;

#[derive(EnumCount, EnumIter, Clone, Copy)]
pub enum StmtVariants {
    LetStatement,
    ConditionalStatement,
    AssignStatement,
    LoopStatement,
}

impl Distribution<StmtVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StmtVariants {
        let choices: Vec<StmtVariants> = StmtVariants::iter().collect();

        let dist = WeightedIndex::new(StmtVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}
