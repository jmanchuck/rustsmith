pub mod safe_ops;

#[cfg(test)]
mod test {
    #[test]
    fn checked_add() {
        assert_eq!(5i32.checked_add(10).unwrap(), 15);
    }
}
