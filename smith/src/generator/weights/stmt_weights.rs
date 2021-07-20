use crate::program::stmt::stmt::StmtVariants;
use rand::{
    distributions::{Distribution, Standard, WeightedIndex},
    Rng,
};
use strum::{EnumCount, IntoEnumIterator};

const STMT_WEIGHTS: [u32; StmtVariants::COUNT] = [2, 0, 1, 2, 0, 0];

impl Distribution<StmtVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StmtVariants {
        let choices: Vec<StmtVariants> = StmtVariants::iter().collect();

        let dist = WeightedIndex::new(&STMT_WEIGHTS).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}
