use rand::{
    distributions::{Distribution, Standard, WeightedIndex},
    Rng,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter};

use super::super::EnumWeights;

#[derive(EnumCount, EnumIter, Clone, Copy)]
pub enum ArithmeticExprVariants {
    Int,
    Binary,
    Var,
    Func,
}

#[derive(EnumCount, EnumIter, Clone, Copy)]
pub enum BoolExprVariants {
    Bool,
    Binary,
    Comparison,
    Negation,
    Var,
    Func,
}

#[derive(EnumCount, EnumIter, Clone, Copy)]
pub enum StructExprVariants {
    Literal,
    Var,
    Func,
}

impl Distribution<ArithmeticExprVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ArithmeticExprVariants {
        let choices: Vec<ArithmeticExprVariants> = ArithmeticExprVariants::iter().collect();

        let dist = WeightedIndex::new(ArithmeticExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}

impl Distribution<BoolExprVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BoolExprVariants {
        let choices: Vec<BoolExprVariants> = BoolExprVariants::iter().collect();

        let dist = WeightedIndex::new(BoolExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}

impl Distribution<StructExprVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StructExprVariants {
        let choices: Vec<StructExprVariants> = StructExprVariants::iter().collect();

        let dist = WeightedIndex::new(StructExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}
