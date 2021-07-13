#![allow(warnings)]

use smith::generate_from_seed;
pub fn generate(count: u64) -> std::io::Result<()> {
    for i in 0..count {
        let code = generate_from_seed(i);
        let path = "../bin/";
        std::fs::write(path, code)?;
    }

    Ok(())
}

pub enum OptLevel {
    ZERO,
    ONE,
    TWO,
    THREE,
    SIZE,
    ZSIZE,
}

impl ToString for OptLevel {
    fn to_string(&self) -> String {
        match self {
            OptLevel::ZERO => String::from("opt0"),
            OptLevel::ONE => String::from("opt1"),
            OptLevel::TWO => String::from("opt2"),
            OptLevel::THREE => String::from("opt3"),
            OptLevel::SIZE => String::from("optS"),
            OptLevel::ZSIZE => String::from("optZ"),
        }
    }
}
