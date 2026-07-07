use std::fs;
use std::path::PathBuf;
use std::process::exit;

const USAGE: &str = "Usage: generated --seed <u64> [--count <u64>] [--out <dir>]
  -s, --seed <u64>    Starting seed (required)
  -c, --count <u64>   Number of programs to generate (default: 1)
  -o, --out <dir>     Output directory (default: ./src/bin)";

fn main() {
    let config = RunConfig::new_from_args();

    if let Err(err) = fs::create_dir_all(&config.out) {
        eprintln!("Failed to create output directory: {}", err);
        exit(1);
    }

    for seed in config.seed..config.seed + config.count {
        let code = smith::generate_from_seed(seed);

        let file_path = config.out.join(format!("{}.rs", RunConfig::as_file_name(seed)));

        match fs::write(&file_path, code.as_str()) {
            Ok(_) => (),
            Err(err) => panic!("Failed to generate, {}", err),
        };
    }
}

pub struct RunConfig {
    seed: u64,
    count: u64,
    out: PathBuf,
}

impl RunConfig {
    pub fn new_from_args() -> Self {
        let mut seed: Option<u64> = None;
        let mut count: u64 = 1;
        let mut out = PathBuf::from("./src/bin");

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-s" | "--seed" => seed = Some(Self::parse_u64_value(&arg, args.next())),
                "-c" | "--count" => count = Self::parse_u64_value(&arg, args.next()),
                "-o" | "--out" => match args.next() {
                    Some(value) => out = PathBuf::from(value),
                    None => Self::usage_error(&format!("Missing value for {}", arg)),
                },
                _ => Self::usage_error(&format!("Unexpected argument: {}", arg)),
            }
        }

        let seed = match seed {
            Some(seed) => seed,
            None => {
                Self::usage_error("Missing required argument: --seed");
            }
        };

        RunConfig { seed, count, out }
    }

    pub fn as_file_name(seed: u64) -> String {
        format!("seed_{}", seed)
    }

    fn parse_u64_value(flag: &str, value: Option<String>) -> u64 {
        let value = match value {
            Some(value) => value,
            None => Self::usage_error(&format!("Missing value for {}", flag)),
        };

        match value.parse::<u64>() {
            Ok(parsed) => parsed,
            Err(_) => Self::usage_error(&format!("Invalid u64 for {}: {}", flag, value)),
        }
    }

    fn usage_error(message: &str) -> ! {
        eprintln!("{}", message);
        eprintln!("{}", USAGE);
        exit(2);
    }
}
