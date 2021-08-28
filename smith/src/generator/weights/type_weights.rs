use crate::program::expr::arithmetic_expr::BinaryOp;
use crate::program::expr::arithmetic_expr::IntValue;
use crate::program::expr::bool_expr::{BoolOp, ComparisonOp};
use crate::program::types::{BorrowTypeID, IntTypeID, TypeIDVariants};

use rand::prelude::SliceRandom;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use strum::IntoEnumIterator;

// Not weighted
impl Distribution<TypeIDVariants> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TypeIDVariants {
        let choices: Vec<TypeIDVariants> = TypeIDVariants::iter().collect();

        *(choices.choose(rng).unwrap())
    }
}

impl Distribution<BorrowTypeID> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BorrowTypeID {
        let choices: Vec<BorrowTypeID> = BorrowTypeID::iter().collect();

        *choices.choose(rng).unwrap()
    }
}

impl Distribution<IntTypeID> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> IntTypeID {
        match rng.gen_range(0..=9) {
            0 => IntTypeID::I8,
            1 => IntTypeID::I16,
            2 => IntTypeID::I32,
            3 => IntTypeID::I64,
            4 => IntTypeID::I128,
            5 => IntTypeID::U8,
            6 => IntTypeID::U16,
            7 => IntTypeID::U32,
            8 => IntTypeID::U64,
            _ => IntTypeID::U128,
        }
    }
}

impl IntValue {
    pub fn rand_from_type<R: Rng>(int_type_id: IntTypeID, rng: &mut R) -> Self {
        match int_type_id {
            IntTypeID::I8 => IntValue::I8({
                let rand = rng.gen::<i8>();
                if rand == i8::MIN {
                    rand + 1
                } else {
                    rand
                }
            }),
            IntTypeID::I16 => IntValue::I16({
                let rand = rng.gen::<i16>();
                if rand == i16::MIN {
                    rand + 1
                } else {
                    rand
                }
            }),
            IntTypeID::I32 => IntValue::I32({
                let rand = rng.gen::<i32>();
                if rand == i32::MIN {
                    rand + 1
                } else {
                    rand
                }
            }),
            IntTypeID::I64 => IntValue::I64({
                let rand = rng.gen::<i64>();
                if rand == i64::MIN {
                    rand + 1
                } else {
                    rand
                }
            }),
            IntTypeID::I128 => IntValue::I128({
                let rand = rng.gen::<i128>();
                if rand == i128::MIN {
                    rand + 1
                } else {
                    rand
                }
            }),
            IntTypeID::U8 => IntValue::U8(rng.gen::<u8>()),
            IntTypeID::U16 => IntValue::U16(rng.gen::<u16>()),
            IntTypeID::U32 => IntValue::U32(rng.gen::<u32>()),
            IntTypeID::U64 => IntValue::U64(rng.gen::<u64>()),
            IntTypeID::U128 => IntValue::U128(rng.gen::<u128>()),
        }
    }
}

impl Distribution<BinaryOp> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BinaryOp {
        let choices: Vec<BinaryOp> = BinaryOp::iter().collect();

        *choices.choose(rng).unwrap()
    }
}

impl Distribution<ComparisonOp> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ComparisonOp {
        match rng.gen_range(0..=5) {
            0 => ComparisonOp::Greater,
            1 => ComparisonOp::GreaterEq,
            2 => ComparisonOp::Less,
            3 => ComparisonOp::LessEq,
            4 => ComparisonOp::Equal,
            _ => ComparisonOp::NotEqual,
        }
    }
}

impl Distribution<BoolOp> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BoolOp {
        match rng.gen_range(0..=2) {
            0 => BoolOp::AND,
            _ => BoolOp::OR,
        }
    }
}
