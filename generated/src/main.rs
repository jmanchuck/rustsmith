use clap::{App, Arg, ArgMatches};
use indicatif::ProgressBar;
use std::fs;
fn main() {
    let config = RunConfig::new_from_args(get_args());

    println!("{:#?}", config);

    let progress_bar = ProgressBar::new(100);

    for seed in config.seed()..config.seed() + config.count() {
        let code = smith::generate_from_seed(seed);

        let file_path = format!("./src/bin/{}.rs", RunConfig::as_file_name(seed));

        match fs::write(file_path, code.as_str()) {
            Ok(_) => (),
            Err(err) => panic!("Failed to generate, {}", err),
        };

        progress_bar.inc((seed - config.seed()) / config.count() as u64);
    }
}

pub fn get_args() -> ArgMatches<'static> {
    App::new("RustSmith")
        .version("0.1.0")
        .author("JJ <jjcheung0000@gmail.com>")
        .about("Rust program generator")
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .takes_value(true)
                .help("Unsigned 64 integer"),
        )
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .takes_value(true)
                .help("Name of file to be generated"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .takes_value(true)
                .help("Number of random programs to generate"),
        )
        .get_matches()
}

#[derive(Debug)]
pub struct RunConfig {
    seed: u64,
    count: u64,
}

impl RunConfig {
    pub fn new(seed: u64, count: u64) -> Self {
        RunConfig { seed, count }
    }

    pub fn new_from_args(args: ArgMatches) -> Self {
        let seed: u64 = RunConfig::parse_seed(&args);
        let count: u64 = RunConfig::parse_count(&args);

        RunConfig { seed, count }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn as_file_name(seed: u64) -> String {
        format!("seed_{}", seed)
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    fn parse_seed(args: &ArgMatches) -> u64 {
        match args.value_of("seed") {
            None => Default::default(),
            Some(seed_str) => match seed_str.parse::<u64>() {
                Err(_) => {
                    println!("Failed to parse seed, using default seed");
                    Default::default()
                }
                Ok(seed_int) => seed_int,
            },
        }
    }

    fn parse_count(args: &ArgMatches) -> u64 {
        match args.value_of("count") {
            None => 1,
            Some(value) => value.parse::<u64>().unwrap_or(1),
        }
    }
}
