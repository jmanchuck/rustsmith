use rand::{prelude::StdRng, SeedableRng};

use crate::generator::main_gen;

pub mod generator;
pub mod program;

pub fn generate_from_seed(seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);

    let warning_macro = String::from("#![allow(warnings)]\n");
    let imports = String::from(
        "use serde::Serialize;\nuse serde_json::Serializer;\nuse runtime::{ops::BitArithmetic, safe_ops::SafeArithmetic};\n",
    );
    let main = main_gen::gen_main(&mut rng);

    let code = format!("{}{}{}", warning_macro, imports, main);

    code
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn same_seed_generates_same_program() {
        for i in 0..50 {
            let mut rng1 = StdRng::seed_from_u64(i);
            let mut rng2 = StdRng::seed_from_u64(i);

            let main1 = main_gen::gen_main(&mut rng1);
            let main2 = main_gen::gen_main(&mut rng2);

            assert_eq!(main1, main2);
        }
    }
}
