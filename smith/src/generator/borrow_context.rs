#![allow(dead_code)]
use std::collections::BTreeMap;

use crate::program::types::BorrowStatus;

#[derive(Clone)]
struct BorrowContext {
    vars: BTreeMap<String, BorrowStatus>,
}

struct BorrowRelation {
    source: String, // Could use an Rc<RefCell<data>> to remove itself
    borrows: Vec<String>,
    mut_borrow: Option<String>,
}
