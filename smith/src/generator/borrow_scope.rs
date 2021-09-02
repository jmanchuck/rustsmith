/// Maintains the borrow information of a single entry
use std::collections::HashSet;

use crate::program::types::BorrowStatus;

#[derive(Clone, Debug)]
pub struct BorrowContext {
    borrow_source: Option<String>,
    borrows: HashSet<String>,
    mut_borrows: HashSet<String>,
    func_mut_borrow: bool,
}

impl BorrowContext {
    pub fn new(source: Option<String>) -> Self {
        BorrowContext {
            borrow_source: source,
            borrows: HashSet::new(),
            mut_borrows: HashSet::new(),
            func_mut_borrow: false,
        }
    }

    pub fn get_borrow_status(&self) -> BorrowStatus {
        if self.is_mut_borrowed() || self.is_func_mut_borrowed() {
            BorrowStatus::MutBorrowed
        } else if self.is_borrowed() {
            BorrowStatus::Borrowed
        } else {
            BorrowStatus::None
        }
    }

    pub fn get_borrow_source(&self) -> Option<String> {
        self.borrow_source.clone()
    }

    pub fn is_mut_borrowed(&self) -> bool {
        self.mut_borrows.len() > 0
    }

    pub fn is_borrowed(&self) -> bool {
        self.borrows.len() > 0
    }

    pub fn can_mut_borrow(&self) -> bool {
        !self.is_mut_borrowed() && self.borrows.len() == 0
    }

    pub fn mut_borrow(&mut self, borrow_var: &String) {
        self.mut_borrows.insert(borrow_var.clone());
    }

    pub fn borrow(&mut self, borrow_var: &String) {
        self.borrows.insert(borrow_var.clone());
    }

    pub fn get_mut_borrows(&self) -> Vec<String> {
        self.mut_borrows.clone().into_iter().collect()
    }

    pub fn get_borrows(&self) -> Vec<String> {
        self.borrows.iter().map(|x| x.to_string()).collect()
    }

    pub fn remove_borrow(&mut self, borrow_var: &String) {
        if self.borrows.contains(borrow_var) && self.mut_borrows.contains(borrow_var) {
            panic!("The same variable has mutably and immutably borrowed from this variable");
        } else if self.borrows.contains(borrow_var) {
            self.borrows.remove(borrow_var);
        } else if self.mut_borrows.contains(borrow_var) {
            self.mut_borrows.remove(borrow_var);
        }
    }

    pub fn func_mut_borrow(&mut self) {
        self.func_mut_borrow = true;
    }

    pub fn is_func_mut_borrowed(&self) -> bool {
        self.func_mut_borrow
    }
}
