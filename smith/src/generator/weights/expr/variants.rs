use crate::rng::{RandGen, Rng, WeightedIndex};

use super::super::EnumWeights;

#[derive(Clone, Copy)]
pub enum ArithmeticExprVariants {
    Int,
    Binary,
    Cast,
    Unary,
    Var,
    Func,
}

impl ArithmeticExprVariants {
    pub const ALL: &'static [Self] = &[
        Self::Int,
        Self::Binary,
        Self::Cast,
        Self::Unary,
        Self::Var,
        Self::Func,
    ];
}

#[derive(Clone, Copy)]
pub enum BoolExprVariants {
    Bool,
    Binary,
    Comparison,
    Negation,
    Var,
    Func,
}

impl BoolExprVariants {
    pub const ALL: &'static [Self] = &[
        Self::Bool,
        Self::Binary,
        Self::Comparison,
        Self::Negation,
        Self::Var,
        Self::Func,
    ];
}

#[derive(Clone, Copy)]
pub enum StructExprVariants {
    Literal,
    Var,
    Func,
}

impl StructExprVariants {
    pub const ALL: &'static [Self] = &[Self::Literal, Self::Var, Self::Func];
}

impl RandGen for ArithmeticExprVariants {
    fn rand_gen<R: Rng>(rng: &mut R) -> ArithmeticExprVariants {
        let dist = WeightedIndex::new(ArithmeticExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        ArithmeticExprVariants::ALL[idx]
    }
}

impl RandGen for BoolExprVariants {
    fn rand_gen<R: Rng>(rng: &mut R) -> BoolExprVariants {
        let dist = WeightedIndex::new(BoolExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        BoolExprVariants::ALL[idx]
    }
}

impl RandGen for StructExprVariants {
    fn rand_gen<R: Rng>(rng: &mut R) -> StructExprVariants {
        let dist = WeightedIndex::new(StructExprVariants::weights()).unwrap();
        let idx = dist.sample(rng);

        StructExprVariants::ALL[idx]
    }
}
