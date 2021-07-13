pub struct NameGenerator {
    curr: u32,
    prefix: String,
}

impl NameGenerator {
    pub fn new(prefix: String) -> Self {
        NameGenerator { curr: 1, prefix }
    }
}

impl Iterator for NameGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let name = format!("{}{}", self.prefix, self.curr.to_string());

        self.curr += 1;

        Some(name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn generate_name() {
        let mut generator = NameGenerator::new(String::from("test_"));
        assert_eq!(generator.next(), Some(String::from("test_1")));
        assert_eq!(generator.next(), Some(String::from("test_2")));
    }
}
