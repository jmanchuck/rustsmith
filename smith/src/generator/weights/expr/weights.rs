use crate::generator::weights::EnumWeights;

use super::variants::{ArithmeticExprVariants, BoolExprVariants, StructExprVariants};

impl EnumWeights for ArithmeticExprVariants {
    fn all() -> &'static [Self] {
        ArithmeticExprVariants::ALL
    }

    fn weight(&self) -> u32 {
        match self {
            ArithmeticExprVariants::Int => 2,
            ArithmeticExprVariants::Binary => 4,
            ArithmeticExprVariants::Cast => 2,
            ArithmeticExprVariants::Unary => 2,
            ArithmeticExprVariants::ArrayIndex => 2,
            ArithmeticExprVariants::Var => 2,
            ArithmeticExprVariants::Func => 2,
        }
    }
}

impl EnumWeights for BoolExprVariants {
    fn all() -> &'static [Self] {
        BoolExprVariants::ALL
    }

    fn weight(&self) -> u32 {
        match self {
            BoolExprVariants::Bool => 1,
            BoolExprVariants::Binary => 4,
            BoolExprVariants::Comparison => 3,
            BoolExprVariants::Negation => 3,
            BoolExprVariants::Var => 2,
            BoolExprVariants::Func => 2,
        }
    }
}

impl EnumWeights for StructExprVariants {
    fn all() -> &'static [Self] {
        StructExprVariants::ALL
    }

    fn weight(&self) -> u32 {
        match self {
            StructExprVariants::Literal => 1,
            StructExprVariants::Var => 2,
            StructExprVariants::Func => 1,
        }
    }
}
