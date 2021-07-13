use rand::{
    distributions::{Distribution, Standard, WeightedIndex},
    Rng,
};
use strum::{EnumCount, IntoEnumIterator};

use crate::program::expr::expr::ArithmeticExprVariants;
use crate::program::expr::struct_expr::StructExprVariants;

// We can modify this const into a static singleton struct and allow sample func to access atomically
const ARITHMETIC_EXPR_WEIGHTS: [u32; ArithmeticExprVariants::COUNT] = [2, 2, 2, 2];
const STRUCT_EXPR_WEIGHTS: [u32; StructExprVariants::COUNT] = [1, 1];

// impl ArithmeticExprVariants {
//     pub fn weight(&self) -> u32 {
//         match self {
//             ArithmeticExprVariants::Int => 5 // global_struct.weight,
//             ArithmeticExprVariants::Binary => todo!(),
//             ArithmeticExprVariants::Var => todo!(),
//             ArithmeticExprVariants::Func => todo!(),
//         }
//     }
// }

impl Distribution<ArithmeticExprVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ArithmeticExprVariants {
        let choices: Vec<ArithmeticExprVariants> = ArithmeticExprVariants::iter().collect();

        let dist = WeightedIndex::new(&ARITHMETIC_EXPR_WEIGHTS).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}

impl Distribution<StructExprVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StructExprVariants {
        let choices: Vec<StructExprVariants> = StructExprVariants::iter().collect();

        let dist = WeightedIndex::new(&STRUCT_EXPR_WEIGHTS).unwrap();
        let idx = dist.sample(rng);

        choices[idx]
    }
}
